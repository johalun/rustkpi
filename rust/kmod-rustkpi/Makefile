#
# Kernel library
#

######## GENERAL VARIABLES

CWD!=pwd
ROOT=${CWD}/..
TARGET_TRIPLE=x86_64-unknown-freebsd-kernel
RUST_SRC?=${ROOT}/rust-src-1.25.0/src
SYSROOTFLAG?=
CRATE_TYPE=lib
OBJDIR?=/tmp/rkpi/objs
LIBDIR?=/tmp/rkpi/rlibs

COMPILERFLAGS?=-C opt-level=3 -C debuginfo=2 -C panic=abort

# TARGET_MACHINE?=192.168.75.68

######## TARGET SETUP

kernel_DIR=${CWD}/kernel
kernel_LIBRS=${kernel_DIR}/lib.rs
kernel_SRCS!=find ${kernel_DIR} -name "*.rs"
kernel_DEPS=core std_unicode alloc alloc_kernel spin

core_DIR=${RUST_SRC}/libcore
core_LIBRS=${core_DIR}/lib.rs
core_SRCS!=find ${core_DIR} -name "*.rs"
core_DEPS=

std_unicode_DIR=${RUST_SRC}/libstd_unicode
std_unicode_LIBRS=${std_unicode_DIR}/lib.rs
std_unicode_SRCS!=find ${std_unicode_DIR} -name "*.rs"
std_unicode_DEPS=core

alloc_DIR=${RUST_SRC}/liballoc
alloc_LIBRS=${alloc_DIR}/lib.rs
alloc_SRCS!=find ${alloc_DIR} -name "*.rs"
alloc_DEPS=core std_unicode

alloc_kernel_DIR=${ROOT}/crates-own/liballoc_kernel
alloc_kernel_LIBRS=${alloc_kernel_DIR}/lib.rs
alloc_kernel_SRCS!=find ${alloc_kernel_DIR} -name "*.rs"
alloc_kernel_DEPS=core liballoc

spin_DIR=${ROOT}/crates-ext/spin-rs
spin_LIBRS=${spin_DIR}/src/lib.rs
spin_SRCS!=find ${spin_DIR} -name "*.rs"
spin_DEPS=core

interpolate_DIR=${ROOT}/crates-ext/interpolate_idents
interpolate_LIBRS=${interpolate_DIR}/src/lib.rs
interpolate_SRCS!=find ${interpolate_DIR} -name "*.rs"
interpolate_DEPS=



######## KERNEL DEPENDENCIES TARGETS

${OBJDIR}/core.o: ${RUST_SRC}/libcore/Cargo.toml
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG} 

###################################

${OBJDIR}/std_unicode.o: ${RUST_SRC}/libstd_unicode/Cargo.toml
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}

###################################

${OBJDIR}/alloc.o: ${RUST_SRC}/liballoc/Cargo.toml
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}

###################################

${OBJDIR}/alloc_kernel.o: ${alloc_kernel_LIBRS}
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}

###################################

${OBJDIR}/rand.o: ${RUST_SRC}/vendor/rand/Cargo.toml
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}

###################################

${OBJDIR}/collections.o: ${RUST_SRC}/libcollections/Cargo.toml
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}

###################################

${OBJDIR}/spin.o: ${spin_SRCS}
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} --cfg 'feature="once"'  --cfg 'feature="const_fn"' \
		--cfg 'feature="rustc_private"' \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}


######## KMOD DEPENDENCIES TARGETS

${OBJDIR}/libinterpolate_idents.so:  ${interpolate_SRCS}
	@echo -e "\033[1m===> Building ${.TARGET:T:R:S/^lib//}\033[0m"
	@rustc --crate-name ${.TARGET:R:T:S/^lib//} \
		${ROOT}/crates-ext/${.TARGET:R:T:S/^lib//}/src/lib.rs \
		--crate-type dylib --emit=dep-info,link -C prefer-dynamic -C opt-level=3 \
		--cap-lints allow --out-dir ${OBJDIR}

${OBJDIR}/kernel.o: ${kernel_SRCS}
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${${.TARGET:T:R:S/$/_LIBRS/}} \
		--crate-type ${CRATE_TYPE} \
		--emit=link,dep-info,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} -L all=${OBJDIR} \
		--target ${TARGET_TRIPLE} -Z force-unstable-if-unmarked \
		${SYSROOTFLAG}


######## KMOD TARGETS

${OBJDIR}/rustkpi_kmod.o: ${OBJDIR}/kernel.o ${OBJDIR}/libinterpolate_idents.so kmod/lib.rs
	@echo -e "\033[1m===> Building ${.TARGET:T:R}\033[0m"
	@rustc --crate-name ${.TARGET:R:T} ${CWD}/kmod/lib.rs --crate-type staticlib \
		--emit=dep-info,link,obj ${COMPILERFLAGS} --out-dir ${OBJDIR} \
		--target ${TARGET_TRIPLE} ${SYSROOTFLAG} -L all=${OBJDIR} \
		-Z external-macro-backtrace 


######## OTHER TARGETS

${OBJDIR}:
	@echo -e "\033[1m===> Creating dir: ${.TARGET}\033[0m"
	@mkdir -p ${OBJDIR}

${LIBDIR}:
	@echo -e "\033[1m===> Creating dir: ${.TARGET}\033[0m"
	@mkdir -p ${LIBDIR}

install-libs: ${LIBDIR}
	@echo -e "\033[1m===> Installing shared rust libs to ${LIBDIR}\033[0m"
	@cp ${OBJDIR}/*.rlib ${LIBDIR}/
	@cp ${OBJDIR}/*.so ${LIBDIR}/

# rustkpi.ko: ${OBJDIR}/kmod.o

# rustkpi-test.ko:
# 	@${MAKE} -C kmod-test

# if_rem.ko:
# 	@${MAKE} -C kmod-e1000

# install-e1000: rustkpi.ko if_rem.ko
# 	@echo -e "\033[1m===> Installing ${.ALLSRC} on target ${TARGET_MACHINE}\033[0m"
# 	@scp rustkpi.ko root@${TARGET_MACHINE}:/boot/modules
# 	@scp kmod-e1000/if_rem.ko root@${TARGET_MACHINE}:/boot/modules

# install-test: rustkpi.ko rustkpi-test.ko
# 	@echo -e "\033[1m===> Installing ${.ALLSRC} on target ${TARGET_MACHINE}\033[0m"
# 	@scp rustkpi.ko root@${TARGET_MACHINE}:/boot/modules
# 	@scp kmod-test/rustkpi-test.ko root@${TARGET_MACHINE}:/boot/modules

# load:
# 	@echo -e "\033[1m===> Loading kmod on target ${TARGET_MACHINE}\033[0m"
# 	@ssh root@${TARGET_MACHINE} kldload rustkpi-test

# load-e1000:
# 	@echo -e "\033[1m===> Loading kmod on target ${TARGET_MACHINE}\033[0m"
# 	@ssh root@${TARGET_MACHINE} kldload if_rem.ko

# unload:
# 	@echo -e "\033[1m===> Unloading kmod on target ${TARGET_MACHINE}\033[0m"
# 	@ssh root@${TARGET_MACHINE} kldunload rustkpi-test

# unload-e1000:
# 	@echo -e "\033[1m===> Unloading kmod on target ${TARGET_MACHINE}\033[0m"
# 	@ssh root@${TARGET_MACHINE} kldunload if_rem.ko

# test:	install-test
# 	@echo -e "\033[1m===> Testing kmod on target ${TARGET_MACHINE}\033[0m"
# 	@${MAKE} load
# 	@sleep 3
# 	@${MAKE} unload

# test-e1000: install-e1000
# 	@echo -e "\033[1m===> Testing if_rem.ko on target ${TARGET_MACHINE}\033[0m"
# 	@${MAKE} load-e1000
# 	@sleep 3
# 	@${MAKE} unload-e1000

clean-all:
	${MAKE} clean
	${MAKE} -C kmod-test clean


######## OBJECT FILES TO BUILD (IN ORDER OF DEPENDENCY)

BUILD_OBJECTS=core std_unicode alloc alloc_kernel spin kernel rustkpi_kmod


######## KMOD VARIABLES

KMOD=	rustkpi

# We need a dummy source file because kmod.mk can't handle empty SRCS
SRCS=	dummy.c

LDFLAGS+=--whole-archive
DEBUG_FLAGS += -g3
#WITH_CTF=1

.for _sub in ${BUILD_OBJECTS}
KMOD_OBJS+=${OBJDIR}/${_sub}.o
.endfor

OBJS:=${OBJDIR}/librustkpi_kmod.a

${OBJDIR}/librustkpi_kmod.a: ${OBJDIR} ${LIBDIR} ${KMOD_OBJS} install-libs

CLEANDIRS+=${OBJDIR} ${LIBDIR}


.include <bsd.kmod.mk>
