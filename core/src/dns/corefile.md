package dns

import (
	"bytes"
	"context"
	"fmt"
	"io/ioutil"
	"net"
	"strconv"
	"strings"
	"text/template"

	v1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/klog/v2"

	"github.com/kubeedge/edgemesh/pkg/apis/config/defaults"
	"github.com/kubeedge/edgemesh/pkg/apis/config/v1alpha1"
	netutil "github.com/kubeedge/edgemesh/pkg/util/net"
)

// copy from https://github.com/kubernetes/dns/blob/1.21.0/cmd/node-cache/app/configmap.go and update
const (
	// stubDomainBlock definisce un modello per la configurazione
	// di un dominio stub nel server DNS.
	// Utilizza la sintassi di templating per inserire i valori delle variabili.
	stubDomainBlock = `{{.DomainName}}:{{.Port}} {
	    bind {{.LocalIP}}              // Specifica l'indirizzo IP locale da utilizzare.
	    cache {{.CacheTTL}}           // Imposta il TTL (Time-To-Live) per la cache delle risposte DNS.
	    errors                        // Abilita la registrazione degli errori.
	    forward . {{.UpstreamServers}} {  // Inoltra le richieste a server upstream specificati.
	        force_tcp                  // Forza l'uso di TCP per le comunicazioni con i server upstream.
	    }
	    {{ .KubernetesPlugin }}       // Includi il blocco del plugin Kubernetes, se presente.
	    log                           // Abilita il logging delle richieste.
	    loop                          // Abilita il controllo del loop.
	    reload                        // Permette il riavvio della configurazione senza downtime.
	}
	`

	// kubernetesPluginBlock definisce un modello per la configurazione
	// di un plugin Kubernetes nel server DNS.
	// Questa configurazione è utilizzata per gestire le richieste DNS per
	// i pod e i servizi in un cluster Kubernetes.
	kubernetesPluginBlock = `kubernetes cluster.local in-addr.arpa ip6.arpa {
	    {{ .APIServer }}               // Specifica l'indirizzo del server API Kubernetes.
	    pods insecure                  // Permette l'accesso non sicuro ai pod.
	    fallthrough in-addr.arpa ip6.arpa // Permette il passaggio delle richieste ad altri handler.
	    ttl {{ .TTL }}                // Imposta il TTL (Time-To-Live) per le risposte DNS.
	}`

	// defaultTTL è il valore predefinito per il TTL delle risposte DNS.
	defaultTTL = 30 // TTL impostato a 30 secondi.

	// defaultUpstreamServer è il server upstream predefinito
	// utilizzato per le richieste DNS.
	defaultUpstreamServer = "/etc/resolv.conf" // Percorso al file di configurazione di risoluzione DNS.

	// kubeSystem è il nome dello spazio dei nomi Kubernetes predefinito
	// per i componenti del sistema.
	kubeSystem = "kube-system" // Nome dello spazio dei nomi per i servizi di sistema.

	// coreDNS è il nome del servizio DNS CoreDNS in Kubernetes.
	coreDNS = "coredns" // Nome del servizio CoreDNS.

	// kubeDNS è il nome del servizio DNS kube-dns in Kubernetes.
	kubeDNS = "kube-dns" // Nome del servizio kube-dns.
)


// copy from https://github.com/kubernetes/dns/blob/1.21.0/cmd/node-cache/app/configmap.go and update
// stubDomainInfo contains all the parameters needed to compute
// a stubDomain block in the Corefile.
// The `stubDomainInfo` type represents information about a domain, including its name, IP address,
// port, cache TTL, upstream servers, and Kubernetes plugin.
// @property {string} DomainName - The `DomainName` property in the `stubDomainInfo` struct represents
// the name of the domain for which the information is being stored.
// @property {string} LocalIP - The `LocalIP` property in the `stubDomainInfo` struct represents the
// local IP address associated with the domain. This IP address is typically used for routing network
// traffic within the local network or system.
// @property {string} Port - The `Port` property in the `stubDomainInfo` struct represents the port
// number associated with the domain. It is used to specify the network port where the domain's
// services can be accessed.
// @property {int} CacheTTL - The `CacheTTL` property in the `stubDomainInfo` struct represents the
// Time To Live (TTL) value for caching DNS records related to the domain. It specifies the duration
// for which the DNS records can be cached before they are considered stale and need to be refreshed.
// @property {string} UpstreamServers - UpstreamServers typically refers to the servers that are
// located upstream from the current server in a network topology. These servers are responsible for
// providing data or services to the current server. In the context of the stubDomainInfo struct, the
// UpstreamServers property likely stores information about the servers that this domain forwards
// @property {string} KubernetesPlugin - The `KubernetesPlugin` property in the `stubDomainInfo` struct
// is used to store information about the Kubernetes plugin associated with the domain. This could
// include details such as the name of the plugin, version, configuration settings, or any other
// relevant information related to Kubernetes integration for the domain.
type stubDomainInfo struct {
	DomainName       string
	LocalIP          string
	Port             string
	CacheTTL         int
	UpstreamServers  string
	KubernetesPlugin string
}

// The `KubernetesPluginInfo` type defines a struct with fields for API server and TTL information.
// @property {string} APIServer - The `APIServer` property in the `KubernetesPluginInfo` struct
// typically stores the URL or hostname of the Kubernetes API server that the plugin will interact
// with.
// @property {int} TTL - Time To Live (TTL) is a property that specifies the maximum amount of time
// that a resource or data is considered valid before it expires or becomes outdated. In the context of
// the `KubernetesPluginInfo` struct, the `TTL` property likely represents the duration for which the
// plugin information
type KubernetesPluginInfo struct {
	APIServer string
	TTL       int
}

// getKubernetesPluginStr genera una stringa di configurazione per il plugin Kubernetes
// utilizzando le informazioni fornite nella configurazione EdgeDNSConfig.
func getKubernetesPluginStr(cfg *v1alpha1.EdgeDNSConfig) (string, error) {
	var apiServer string // Variabile per memorizzare la configurazione dell'API server.

	// Controlla se il campo Master dell'oggetto KubeAPIConfig non è vuoto.
	if cfg.KubeAPIConfig.Master != "" {
		// Crea una stringa per l'endpoint dell'API server e la aggiunge a apiServer.
		endpointStr := fmt.Sprintf("endpoint %s", cfg.KubeAPIConfig.Master)
		apiServer += endpointStr
	}

	// Controlla se il campo KubeConfig dell'oggetto KubeAPIConfig non è vuoto.
	if cfg.KubeAPIConfig.KubeConfig != "" {
		// Crea una stringa per la configurazione kubeconfig.
		kubeConfigStr := fmt.Sprintf("kubeconfig %s \"\"", cfg.KubeAPIConfig.KubeConfig)
		// Se apiServer è vuoto, aggiunge kubeConfigStr direttamente.
		// Altrimenti, aggiunge kubeConfigStr con un rientro per la formattazione.
		if apiServer == "" {
			apiServer += kubeConfigStr
		} else {
			apiServer += "\n        " + kubeConfigStr
		}
	}

	// Crea un'istanza di KubernetesPluginInfo per memorizzare le informazioni necessarie per il template.
	info := &KubernetesPluginInfo{
		APIServer: apiServer, // Imposta il campo APIServer.
		TTL:       defaultTTL, // Imposta il campo TTL al valore predefinito.
	}

	var tpl bytes.Buffer // Buffer per costruire la stringa finale del template.
	// Crea un nuovo template a partire dal blocco di configurazione del plugin Kubernetes.
	tmpl, err := template.New("kubernetesPluginBlock").Parse(kubernetesPluginBlock)
	if err != nil {
		// Restituisce un errore se il parsing del template fallisce.
		return "", fmt.Errorf("failed to parse kubernetesPluginBlock template, err : %w", err)
	}

	// Esegue il template con le informazioni fornite, scrivendo il risultato nel buffer tpl.
	if err := tmpl.Execute(&tpl, *info); err != nil {
		// Restituisce un errore se l'esecuzione del template fallisce.
		return "", fmt.Errorf("failed to create kubernetesPlugin template, err : %w", err)
	}

	// Restituisce la stringa finale generata dal template.
	return tpl.String(), nil
}

// getStubDomainStr genera una stringa di configurazione per i domini stub
// utilizzando una mappa di domini e informazioni stubDomainInfo.
func getStubDomainStr(stubDomainMap map[string][]string, info *stubDomainInfo) (string, error) {
	var tpl bytes.Buffer // Buffer per costruire la stringa finale del template.
	// Itera su ciascun dominio nella mappa di domini stub.
	for domainName, servers := range stubDomainMap {
		// Crea un nuovo template a partire dal blocco di configurazione del dominio stub.
		tmpl, err := template.New("stubDomainBlock").Parse(stubDomainBlock)
		if err != nil {
			// Restituisce un errore se il parsing del template fallisce.
			return "", fmt.Errorf("failed to parse stubDomainBlock template, err : %w", err)
		}

		// Imposta il nome del dominio e i server upstream nell'oggetto info.
		info.DomainName = domainName
		info.UpstreamServers = strings.Join(servers, " ") // Unisce gli upstream servers in una stringa.

		// Esegue il template con le informazioni fornite, scrivendo il risultato nel buffer tpl.
		if err := tmpl.Execute(&tpl, *info); err != nil {
			// Restituisce un errore se l'esecuzione del template fallisce.
			return "", fmt.Errorf("failed to create stubDomain template, err : %w", err)
		}
	}

	// Restituisce la stringa finale generata dai template per i domini stub.
	return tpl.String(), nil
}

// copy from https://github.com/kubernetes/dns/blob/1.21.0/cmd/node-cache/app/configmap.go and update
// UpdateCorefile aggiorna il file di configurazione CoreDNS in base alle impostazioni
// specificate nella configurazione EdgeDNSConfig e all'interfaccia Kubernetes fornita.
func UpdateCorefile(cfg *v1alpha1.EdgeDNSConfig, kubeClient kubernetes.Interface) error {
	// Ottiene l'indirizzo IP dell'interfaccia di rete specificata nella configurazione.
	ListenIP, err := netutil.GetInterfaceIP(cfg.ListenInterface)
	if err != nil {
		// Restituisce un errore se non riesce a ottenere l'indirizzo IP.
		return err
	}

	// Imposta i valori predefiniti per cacheTTL e upstreamServers.
	cacheTTL := defaultTTL
	upstreamServers := []string{defaultUpstreamServer}

	// Ottiene la stringa di configurazione del plugin Kubernetes.
	kubernetesPlugin, err := getKubernetesPluginStr(cfg)
	if err != nil {
		// Restituisce un errore se la generazione della stringa del plugin fallisce.
		return err
	}

	// Se il caching DNS è abilitato nella configurazione.
	if cfg.CacheDNS.Enable {
		// Resetta la lista degli upstream servers.
		upstreamServers = []string{}
		// Se l'auto-rilevamento è abilitato, aggiunge gli upstream servers rilevati dal cluster.
		if cfg.CacheDNS.AutoDetect {
			upstreamServers = append(upstreamServers, detectClusterDNS(kubeClient)...)
		}

		// Aggiunge gli upstream servers specificati nella configurazione, ignorando le righe vuote.
		for _, server := range cfg.CacheDNS.UpstreamServers {
			server = strings.TrimSpace(server) // Rimuove spazi bianchi all'inizio e alla fine.
			if server == "" { // Ignora server vuoti.
				continue
			}
			// Verifica se l'indirizzo del server è valido prima di aggiungerlo.
			if isValidAddress(server) {
				upstreamServers = append(upstreamServers, server)
			} else {
				// Registra un messaggio di errore per indirizzi non validi.
				klog.Errorf("Invalid address: %s", server)
			}
		}

		// Rimuove eventuali indirizzi duplicati dalla lista degli upstream servers.
		upstreamServers = removeDuplicate(upstreamServers)
		if len(upstreamServers) == 0 {
			// Restituisce un errore se non ci sono upstream servers validi.
			return fmt.Errorf("failed to get nodelocal dns upstream servers")
		}
		// Logga gli upstream servers trovati.
		klog.Infof("nodelocal dns upstream servers: %v", upstreamServers)
		cacheTTL = cfg.CacheDNS.CacheTTL // Aggiorna il TTL della cache.
		// Disabilita il plugin Kubernetes di CoreDNS.
		kubernetesPlugin = ""
	}

	// Crea una mappa per i domini stub e aggiunge il dominio radice "." con gli upstream servers.
	stubDomainMap := make(map[string][]string)
	stubDomainMap["."] = upstreamServers
	// Genera la stringa di configurazione per il dominio stub.
	stubDomainStr, err := getStubDomainStr(stubDomainMap, &stubDomainInfo{
		LocalIP:          ListenIP.String(), // Imposta l'IP locale.
		Port:             fmt.Sprintf("%d", cfg.ListenPort), // Imposta la porta di ascolto.
		CacheTTL:         cacheTTL, // Imposta il TTL della cache.
		KubernetesPlugin: kubernetesPlugin, // Imposta la stringa del plugin Kubernetes.
	})
	if err != nil {
		// Restituisce un errore se la generazione della stringa del dominio stub fallisce.
		return err
	}

	// Crea un buffer per la nuova configurazione.
	newConfig := bytes.Buffer{}
	newConfig.WriteString(stubDomainStr) // Scrive la stringa di configurazione del dominio stub nel buffer.
	// Scrive il buffer nel file di configurazione CoreDNS temporaneo.
	if err := ioutil.WriteFile(defaults.TempCorefilePath, newConfig.Bytes(), 0666); err != nil {
		// Restituisce un errore se il salvataggio del file fallisce.
		return fmt.Errorf("failed to write %s, err %w", defaults.TempCorefilePath, err)
	}

	// Restituisce nil se l'aggiornamento è avvenuto con successo.
	return nil
}

// detectClusterDNS rileva automaticamente gli indirizzi IP dei servizi DNS nel cluster Kubernetes.
// Restituisce un array di stringhe contenente gli indirizzi dei server DNS trovati.
func detectClusterDNS(kubeClient kubernetes.Interface) (servers []string) {
	// Recupera il servizio CoreDNS dal namespace kube-system.
	coredns, err := kubeClient.CoreV1().Services(kubeSystem).Get(context.Background(), coreDNS, metav1.GetOptions{})
	if err == nil && coredns.Spec.ClusterIP != v1.ClusterIPNone {
		// Se la richiesta ha avuto successo e il ClusterIP di CoreDNS è valido,
		// aggiunge l'indirizzo IP alla lista dei server.
		servers = append(servers, coredns.Spec.ClusterIP)
	}

	// Recupera il servizio kube-dns dal namespace kube-system.
	kubedns, err := kubeClient.CoreV1().Services(kubeSystem).Get(context.Background(), kubeDNS, metav1.GetOptions{})
	if err == nil && kubedns.Spec.ClusterIP != v1.ClusterIPNone {
		// Se la richiesta ha avuto successo e il ClusterIP di kube-dns è valido,
		// aggiunge l'indirizzo IP alla lista dei server.
		servers = append(servers, kubedns.Spec.ClusterIP)
	}

	// Recupera tutti i servizi con il label k8s-app=kube-dns nel namespace kube-system.
	kubeDNSList, err := kubeClient.CoreV1().Services(kubeSystem).List(context.Background(), metav1.ListOptions{LabelSelector: "k8s-app=kube-dns"})
	if err == nil {
		// Se la richiesta ha avuto successo, itera attraverso i servizi trovati.
		for _, item := range kubeDNSList.Items {
			// Aggiunge il ClusterIP del servizio alla lista dei server se è valido.
			if item.Spec.ClusterIP != v1.ClusterIPNone {
				servers = append(servers, item.Spec.ClusterIP)
			}
		}
	}

	// Rimuove eventuali indirizzi duplicati dalla lista dei server.
	servers = removeDuplicate(servers)

	// Se non sono stati trovati server, logga un messaggio di errore.
	if len(servers) == 0 {
		klog.Errorf("Unable to automatically detect cluster dns. Do you have coredns or kube-dns installed in your cluster?")
	} else {
		// Logga gli indirizzi dei server DNS trovati.
		klog.Infof("Automatically detect cluster dns: %v", servers)
	}

	// Restituisce la lista degli indirizzi dei server DNS trovati.
	return servers
}

/* 
##############################################################################################################################
###############################################################################################################################
################################################ U T I L I T I E S  ############################################################
################################################################################################################################
################################################################################################################################
*/

// isValidAddress verifica se una stringa rappresenta un indirizzo valido,
// che può essere solo un indirizzo IP o una coppia indirizzo:porta.
func isValidAddress(addr string) bool {
	// Divide l'indirizzo in base ai due punti ":".
	items := strings.Split(addr, ":")
	if len(items) == 1 {
		// Se ci sono solo due elementi, verifica se il primo è un IP valido.
		return isValidIP(items[0])
	} else if len(items) == 2 {
		// Se ci sono due elementi, verifica se il primo è un IP valido e il secondo è una porta valida.
		return isValidIP(items[0]) && isValidPort(items[1])
	}
	// Se ci sono più di due elementi, l'indirizzo non è valido.
	return false
}

// isValidIP verifica se una stringa rappresenta un indirizzo IP valido.
// Restituisce true se l'indirizzo è valido, false altrimenti.
func isValidIP(ip string) bool {
	// Usa net.ParseIP per analizzare l'indirizzo IP e controlla se non è nil.
	return net.ParseIP(ip) != nil
}

// isValidPort verifica se una stringa rappresenta un numero di porta valido.
// Restituisce true se la porta è compresa tra 1 e 65535, false altrimenti.
func isValidPort(port string) bool {
	// Converte la stringa della porta in un numero intero.
	pnum, err := strconv.Atoi(port)
	if err != nil {
		// Se la conversione fallisce, restituisce false.
		return false
	}
	// Verifica se il numero della porta è nel range valido.
	if 0 < pnum && pnum < 65536 {
		return true
	}
	return false
}

// removeDuplicate rimuove i duplicati da una slice di stringhe.
// Restituisce una nuova slice contenente solo valori unici.
func removeDuplicate(ss []string) []string {
	// Inizializza una slice vuota per il risultato.
	ret := make([]string, 0)
	// Utilizza una mappa per tenere traccia dei valori già visti.
	tmp := make(map[string]struct{})
	for _, s := range ss {
		// Controlla se l'elemento non è già stato aggiunto alla mappa.
		if _, ok := tmp[s]; !ok {
			// Se non è presente, aggiungilo al risultato e alla mappa.
			ret = append(ret, s)
			tmp[s] = struct{}{}
		}
	}
	// Restituisce la slice risultante con solo valori unici.
	return ret
}
