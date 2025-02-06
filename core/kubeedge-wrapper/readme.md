This folder contains a KubeEdge wrapper designed to facilitate seamless interaction and integration with the KubeEdge framework. It provides essential functionality to enable access to both EdgeHub and CloudCore, the core components of KubeEdge. The wrapper serves as a bridge, simplifying the management and communication between edge nodes and the cloud.

Commands:  
Reference: https://go.dev/doc/tutorial/compile-install  
    - go mod tidy  
    - go build -o libkubeedge.so -buildmode=c-shared kubeedge.go  
    - go run _filename_
    - go get _libname_  