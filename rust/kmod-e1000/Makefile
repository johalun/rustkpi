#
# Rust port of the e1000 NIC driver
#

######## GENERAL VARIABLES

CWD!=pwd
ROOT=../../
HOST_TRIPLE=x86_64-unknown-freebsd
TARGET_TRIPLE=x86_64-unknown-freebsd-kernel
RUST_SRC?=${ROOT}/rust-src-1.25.0/src
SYSROOTFLAG?=
OBJDIR?=/tmp/rkpi/objs
LIBDIR?=/tmp/rkpi/rlibs
MODULE_SRCS!=find ${CWD}/src -name "*.rs"
COMPILERFLAGS?=-C opt-level=3 -C debuginfo=2 -C panic=abort


######## KMOD VARIABLES

KMOD=	if_rem

SRCS= 	rust_em_if.c rust_em_wrapper.c
SRCS+=	device_if.h bus_if.h pci_if.h opt_ddb.h opt_inet.h opt_inet6.h ifdi_if.h

OBJS=	${OBJDIR}/rust_em.o

CLEANFILES+=${OBJDIR}/librust_em.a
DEBUG_FLAGS += -g3
#WITH_CTF=1


######## TARGETS

${OBJDIR}:
	@echo -e "\033[1m===> Create folder ${.TARGET:T:R}\033[0m"
	@mkdir -p ${OBJDIR}

${OBJDIR}/rust_em.o: ${OBJDIR} ${MODULE_SRCS} ${LIBDIR}/libkernel.rlib
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${CWD}/src/lib.rs --crate-type staticlib \
		--emit=dep-info,link,obj ${COMPILERFLAGS} --color always -L all=${LIBDIR} \
		--target ${TARGET_TRIPLE} ${SYSROOTFLAG} --out-dir ${OBJDIR} 

.include <bsd.kmod.mk>
