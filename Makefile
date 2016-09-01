test:
	docker run -it --rm \
		-v $(PWD):/source \
	  jimmycuadra/rust \
		cargo test
