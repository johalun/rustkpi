#
#
#

BINDGEN_OPTIONS= #--ctypes-prefix ::std::os::raw
BINDGEN_FLAGS= 	--impl-debug --with-derive-default #--use-core
CLANG_ARGS= 	-O2 -pipe -fno-strict-aliasing -D_KERNEL -DKLD_MODULE -nostdinc -I. -I/usr/include \
		-fno-common -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -mcmodel=kernel \
		-mno-red-zone -mno-mmx -mno-sse -mno-aes -mno-avx -msoft-float \
		-fno-asynchronous-unwind-tables -ffreestanding -fwrapv -fstack-protector \
		-D__printf__=__freebsd_kprintf__ -fdiagnostics-show-option -std=iso9899:1999 \
		-I./include
LIBCLANG_PATH=	/usr/local/llvm50/lib
OUTDIR= 	src/sys
HEADERS!= 	find headers/ -name "*.h"
BINDINGS= 	${HEADERS:R:T:S/^headers-/src\/sys\//g:S/$/_sys.rs/g}

all:	bindgen ${BINDINGS} 

bindgen:
	@echo -e "\033[1m"
	@bindgen -V
	@echo -e "\033[0m"

${BINDINGS}: headers/headers-${.TARGET:R:T:S/_sys//g}.h
	@echo -e "\033[1m===> Generating bindings for ${.TARGET:R:T:S/_sys//g}.h\033[0m"
	@LIBCLANG_PATH=${LIBCLANG_PATH} bindgen ${BINDGEN_FLAGS} ${BINDGEN_OPTIONS} \
		--output ${.TARGET} ${.ALLSRC} -- ${CLANG_ARGS} 

test:
	@cargo test

clean:
	@rm -f ${OUTDIR}/*_sys.rs
	@cargo clean
