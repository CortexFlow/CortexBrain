## CIDR: Classless Inter-Domain Routing

**CIDR** (Classless Inter-Domain Routing, routing inter-dominio senza classi) è un metodo per rappresentare e gestire indirizzi IP e reti su Internet o su una rete privata. È utilizzato per ottimizzare l'allocazione degli indirizzi IP e per permettere una più efficiente aggregazione delle rotte.

### Sintassi di CIDR

Un indirizzo CIDR ha il formato:


Dove:
- **IP_ADDRESS**: è l'indirizzo IP di base (può essere un IPv4 come `192.168.1.0` o un IPv6 come `2001:0db8::`).
- **PREFIX_LENGTH**: è un numero che indica quanti bit dell'indirizzo IP sono usati per identificare la rete (notazione "slash", come `/24`).

### Esempio di CIDR in IPv4

Prendiamo `192.168.1.0/24` come esempio:
- `192.168.1.0` è l'indirizzo di rete.
- `/24` significa che i primi 24 bit dell'indirizzo sono riservati per identificare la rete, lasciando i restanti 8 bit per identificare i dispositivi (host) nella rete.  
  Questo permette di avere fino a 256 indirizzi IP (da `192.168.1.0` a `192.168.1.255`), di cui i primi e gli ultimi sono riservati rispettivamente per l'indirizzo di rete e il broadcast.

### Come funziona il CIDR

CIDR permette di "aggregare" e ridurre le rotte. Invece di dividere gli indirizzi in classi (A, B, C) come si faceva in passato, il CIDR permette di assegnare blocchi di indirizzi con una granularità variabile. 

Ad esempio, se una rete usa `192.168.1.0/24`, può essere suddivisa in sottoreti come:
- `192.168.1.0/26` (64 indirizzi)
- `192.168.1.64/26` (64 indirizzi)
- `192.168.1.128/26` (64 indirizzi)
- `192.168.1.192/26` (64 indirizzi)

### Validazione di un CIDR

Un CIDR è valido se:
- L'indirizzo IP è un formato valido.
- Il prefisso è un numero accettabile per IPv4 (0-32) o IPv6 (0-128).

### Utilizzo di CIDR

Il CIDR è fondamentale in molti contesti, tra cui:
- **Reti private**: per organizzare indirizzi IP interni senza usare indirizzi pubblici.
- **Firewall e sicurezza**: per definire intervalli di IP autorizzati o bloccati.
- **Configurazione di reti virtuali**: in ambienti cloud, per assegnare intervalli IP a reti isolate.

Quindi, in un contesto di rete, l’uso delle CIDR permette di configurare e gestire facilmente l’accesso e l’isolamento delle risorse basato su indirizzi IP.
