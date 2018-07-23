#
# Hello World kernel module for rustkpi.
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
COMPILERFLAGS?=-C opt-level=3 -C debuginfo=2 -C panic=abort -C link-dead-code


######## KMOD VARIABLES

KMOD=	rustkpi-hello
# We need a dummy source file because kmod.mk can't handle empty SRCS
SRCS=	dummy.c
OBJS=	${OBJDIR}/hello.o

CLEANFILES+=${OBJDIR}/libhello.a
DEBUG_FLAGS += -g3


######## TARGETS

${OBJDIR}:
	@echo -e "\033[1m===> Create folder ${.TARGET}\033[0m"
	@mkdir -p ${OBJDIR}

${OBJDIR}/hello.o: ${OBJDIR} ${MODULE_SRCS} ${LIBDIR}/libkernel.rlib
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${CWD}/src/lib.rs --crate-type staticlib \
		--emit=dep-info,link,obj -C link-args=--export-dynamic \
		-L all=${LIBDIR} ${COMPILERFLAGS} \
		--target ${TARGET_TRIPLE} ${SYSROOTFLAG} --out-dir ${OBJDIR} 

.include <bsd.kmod.mk>
