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
	docker save -o rmj-be.docker g3n1k/rmj-be

scp:
	scp file_docker rocky37:/home/rocky/workspace/TMP/


#### 
build-manual:
	docker build -f Dockerfile.manual -t g3n1k/rmj-be-manual .

save-manual:
	docker save -o rmj-be-manual.docker g3n1k/rmj-be-manual

scp-manual:
	scp rmj-be-manual.docker rocky37:/home/rocky/workspace/rmj/

manual: build-manual save-manual scp-manual

### 
build-debug:
	docker build -f Dockerfile.debug -t g3n1k/rmj-be-debug .

save-debug:
	docker save -o rmj-be-debug.docker g3n1k/rmj-be-debug

scp-debug:
	scp rmj-be-debug.docker rocky37:/home/rocky/workspace/rmj/

debug: build-debug save-debug scp-debug
