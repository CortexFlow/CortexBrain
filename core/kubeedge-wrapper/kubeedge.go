package main

/*
#include <stdlib.h>
*/
import "C"

import (
	"fmt"

	"github.com/kubeedge/beehive/pkg/core"
	"github.com/kubeedge/edgemesh/pkg/apis/config/v1alpha1"
)

/* Thanks to KubeEdge interface */
type EdgeDNS struct {
	Config *v1alpha1.EdgeDNSConfig
}
func (e *EdgeDNS) Name() string {
	return "EdgeDNS"
}

func (e *EdgeDNS) Group() string {
	return "edgemesh"
}

func (e *EdgeDNS) Enable() bool {
	fmt.Println("EdgeDNS module enabled")
	return true
}

func (e *EdgeDNS) Start() {
	fmt.Println("EdgeDNS module started")
}

func getDefaultEdgeCoreConfig() *EdgeDNS {
	return &EdgeDNS{
		Config: &v1alpha1.EdgeDNSConfig{
			//TODO: create a custom config file and replace the the default config
			//TODO: Default config only exists for testing purpouses
		},
	}
}

//export Register
func Register(configPath *C.char) *C.char {

	/* Workflow: inizialize KubeEdge interface with default config files 
		1. inizialize the GoconfigPath
		2. load the default config path
		3. Register the configuration using the Beehive Core -> KubeEdge Core
		*/

	goConfigPath := C.GoString(configPath)

	//log message
	fmt.Println("Initializing KubeEdge with config path:", goConfigPath)

	// load the config-->default in this case
	config := getDefaultEdgeCoreConfig()

	// log message
	fmt.Printf("Loaded default configuration: %+v\n", config)

	// Register the configuration using the Beehive core
	core.Register(config)

	return C.CString("KubeEdge initialized successfully!")
}

func main() {
	// Funzione placeholder per test e debugging
	fmt.Println("KubeEdge Wrapper is ready.")
}
