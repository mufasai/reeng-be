img = g3n1k/reenginering-tools-be
imgpre ?= reenginering-tools-be-pre
file_docker ?= $(imgpre).docker

build-pre:
	docker build -f Dockerfile.pre -t $(imgpre) .

save-pre:
	docker save -o $(imgpre).docker $(imgpre)

build-post:
	docker build -f Dockerfile.post -t $(img) .

save:
	docker save -o $(file_docker) g3n1k/rmj-be

scp:
	scp $(file_docker) rocky37:/home/rocky/tmp/

