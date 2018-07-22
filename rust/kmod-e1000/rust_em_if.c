
#include <sys/types.h>
#include <sys/param.h>
#include <sys/systm.h>
#include <sys/lock.h>
#include <sys/mutex.h>
#include <sys/mbuf.h>
#include <sys/protosw.h>
#include <sys/socket.h>
#include <sys/malloc.h>
#include <sys/kernel.h>
#include <sys/bus.h>
#include <machine/bus.h>
#include <sys/rman.h>
#include <machine/resource.h>
#include <vm/vm.h>
#include <vm/pmap.h>
#include <machine/clock.h>
#include <dev/pci/pcivar.h>
#include <dev/pci/pcireg.h>

#include "c-src/e1000_hw.h"
#include "pci_if.h"

int rust_pci_find_cap(device_t dev, int capability, int *capreg);
u_int32_t rust_pci_read_config(device_t dev, device_t child, int reg, int width);
u_int32_t rust_pci_get_vendor(device_t dev);
u_int32_t rust_pci_get_subvendor(device_t dev);
u_int32_t rust_pci_get_device(device_t dev);
u_int32_t rust_pci_get_subdevice(device_t dev);
u_int16_t rust_bus_space_read_2(bus_space_tag_t tag, bus_space_handle_t handle, bus_size_t offset);
u_int32_t rust_bus_space_read_4(bus_space_tag_t tag, bus_space_handle_t handle, bus_size_t offset);
void rust_bus_space_write_2(bus_space_tag_t tag, bus_space_handle_t handle, bus_size_t offset, u_int16_t value);
void rust_bus_space_write_4(bus_space_tag_t tag, bus_space_handle_t handle, bus_size_t offset, u_int32_t value);
void rust_usec_delay(int usecs);

/*
 * Wrap troublesome C pre-processor macros in functions that are easy to 
 * call from Rust.
 */
int
rust_pci_find_cap(device_t dev, int capability, int *capreg)
{
    return (PCI_FIND_CAP(device_get_parent(dev), dev, capability, capreg));
}

u_int32_t
rust_pci_read_config(device_t dev, device_t child, int reg, int width)
{
	return PCI_READ_CONFIG(dev, child, reg, width);
}

u_int32_t
rust_pci_get_vendor(device_t dev) {
	return pci_get_vendor(dev);
}

u_int32_t
rust_pci_get_subvendor(device_t dev) {
	return pci_get_subvendor(dev);
}

u_int32_t
rust_pci_get_device(device_t dev) {
	return pci_get_device(dev);
}

u_int32_t
rust_pci_get_subdevice(device_t dev) {
	return pci_get_subdevice(dev);
}

u_int16_t
rust_bus_space_read_2(bus_space_tag_t tag, bus_space_handle_t handle,
    bus_size_t offset)
{
	return bus_space_read_2(tag, handle, offset);
}

u_int32_t
rust_bus_space_read_4(bus_space_tag_t tag, bus_space_handle_t handle,
    bus_size_t offset)
{
	return bus_space_read_4(tag, handle, offset);
}

void
rust_bus_space_write_2(bus_space_tag_t tag, bus_space_handle_t handle,
    bus_size_t offset, u_int16_t value) {
	bus_space_write_2(tag, handle, offset, value);
}

void
rust_bus_space_write_4(bus_space_tag_t tag, bus_space_handle_t handle,
    bus_size_t offset, u_int32_t value) {
	bus_space_write_4(tag, handle, offset, value);
}

void
rust_usec_delay(int usecs) {
	DELAY(usecs);
}
