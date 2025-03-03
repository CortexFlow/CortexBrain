#!/bin/bash

echo "Welcome to the CortexFlow tools"
echo "Checking pre-requisites for developers"
echo 

echo "Checking Docker installation..."
if which docker >/dev/null 2>&1; then
    echo "✅ Docker is installed."
else
    echo "❌ Docker is NOT installed."
fi
sleep 1

echo 
echo "Checking Minikube installation..."
if which minikube >/dev/null 2>&1; then
    echo "✅ Minikube is installed."
else
    echo "❌ Minikube is NOT installed."
fi
sleep 1

echo 

echo "Checking Node.js installation..."
if which node >/dev/null 2>&1; then
    echo "✅ Node.js is installed."
else
    echo "Node.js is NOT installed."
fi
sleep 1

echo 

echo "Checking npm installation..."
if which npm >/dev/null 2>&1; then
    echo "✅ npm is installed."
else
    echo "❌ npm is NOT installed."
fi
