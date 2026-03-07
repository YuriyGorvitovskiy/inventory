.PHONY: run build docker-build docker-deploy docker-stop docker-stop-all k8s-apply k8s-delete

run:
	cargo run -p inventory-core

build:
	cargo build -p inventory-core

docker-build:
	docker build -t inventory-core:dev -f services/inventory-core/Dockerfile .

docker-deploy:
	./scripts/docker-pipeline.sh

docker-stop:
	docker rm -f inventory-core || true

docker-stop-all:
	docker rm -f inventory-core inventory-postgres || true

k8s-apply:
	kubectl apply -k deploy/k8s/base

k8s-delete:
	kubectl delete -k deploy/k8s/base
