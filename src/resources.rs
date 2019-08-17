// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Structs to manage device resources.
//!
//! The high level flow of resource management among the VMM, the device manager, and the device
//! is as below:
//! 1) the VMM creates a new device object.
//! 2) the VMM asks the new device object for its resource requirements.
//! 3) the VMM allocates resources for the device object according to resource requirements.
//! 4) the VMM passes the allocated resources to the device object.
//! 5) the VMM registers the new device onto corresponding device managers according the allocated
//!    resources.

use std::{u16, u32, u64};

/// Enumeration describing a device's resource requirements.
pub enum ResourceConstraint {
    /// Request for an IO Port address range.
    PioAddress {
        /// Allocate resource within the range [`min`, `max`] if `min` is not u16::MAX.
        min: u16,
        /// Allocate resource within the range [`min`, `max`] if `max` is not zero.
        max: u16,
        /// Alignment for the allocated address.
        align: u16,
        /// Size for the allocated address range.
        size: u16,
    },
    /// Request for a Memory Mapped IO address range.
    MmioAddress {
        /// Allocate resource within the range [`min`, `max`] if 'min' is not u64::MAX.
        min: u64,
        /// Allocate resource within the range [`min`, `max`] if 'max' is not zero.
        max: u64,
        /// Alignment for the allocated address.
        align: u64,
        /// Size for the allocated address range.
        size: u64,
    },
    /// Request for a legacy IRQ.
    LegacyIrq {
        /// Reserve the preallocated IRQ if it's not u32::MAX.
        fixed: u32,
    },
    /// Request for PCI MSI IRQs.
    PciMsiIrq {
        /// Number of Irqs to allocate.
        size: u32,
    },
    /// Request for PCI MSIx IRQs.
    PciMsixIrq {
        /// Number of Irqs to allocate.
        size: u32,
    },
    /// Request for generic IRQs.
    GenericIrq {
        /// Number of Irqs to allocate.
        size: u32,
    },
    /// Request for KVM mem_slot indexes to map memory into the guest.
    KvmMemSlot {
        /// Allocate the specified kvm memory slot index if it's not u32::MAX.
        fixed: u32,
        /// Number of slots to allocate.
        size: u32,
    },
}

impl ResourceConstraint {
    /// Create a new Mmio address request object with default configuration.
    pub fn new_pio(size: u16) -> Self {
        ResourceConstraint::PioAddress {
            min: u16::MAX,
            max: 0,
            align: 0x1,
            size,
        }
    }

    /// Create a new Mmio address request object with default configuration.
    pub fn new_mmio(size: u64) -> Self {
        ResourceConstraint::MmioAddress {
            min: u64::MAX,
            max: 0,
            align: 0x1000,
            size,
        }
    }

    /// Create a new legacy IRQ request object with default configuration.
    pub fn new_legacy_irq() -> Self {
        ResourceConstraint::LegacyIrq { fixed: u32::MAX }
    }

    /// Create a new KVM memory slot request object with default configuration.
    pub fn new_kvm_mem_slot(size: u32) -> Self {
        ResourceConstraint::KvmMemSlot {
            fixed: u32::MAX,
            size,
        }
    }
}

/// Type of Message Singaled Interrupt
#[derive(Copy, Clone, PartialEq)]
pub enum MsiIrqType {
    /// PCI MSI IRQ numbers.
    PciMsi,
    /// PCI MSIx IRQ numbers.
    PciMsix,
    /// Generic MSI IRQ numbers.
    GenericMsi,
}

/// Enumeration for device resources.
#[allow(missing_docs)]
#[derive(Clone)]
pub enum Resource {
    /// IO Port address range.
    PioAddressRange { base: u16, size: u16 },
    /// Memory Mapped IO address range.
    MmioAddressRange { base: u64, size: u64 },
    /// Legacy IRQ number.
    LegacyIrq(u32),
    /// Message Signaled Interrupt
    MsiIrq {
        ty: MsiIrqType,
        base: u32,
        size: u32,
    },
    /// Network Interface Card MAC address.
    MacAddresss(String),
    /// KVM memslot index.
    KvmMemSlot(u32),
}

/// Newtype to store a set of device resources.
#[derive(Default, Clone)]
pub struct DeviceResources(Vec<Resource>);

impl DeviceResources {
    /// Create a container object to store device resources.
    pub fn new() -> Self {
        DeviceResources(Vec::new())
    }

    /// Append a device resource to the container object.
    pub fn append(&mut self, entry: Resource) {
        self.0.push(entry);
    }

    /// Get the IO port address resources.
    pub fn get_pio_address_ranges(&self) -> Vec<(u16, u16)> {
        let mut vec = Vec::new();
        for entry in self.0.iter().as_ref() {
            if let Resource::PioAddressRange { base, size } = entry {
                vec.push((*base, *size));
            }
        }
        vec
    }

    /// Get the Memory Mapped IO address resources.
    pub fn get_mmio_address_ranges(&self) -> Vec<(u64, u64)> {
        let mut vec = Vec::new();
        for entry in self.0.iter().as_ref() {
            if let Resource::MmioAddressRange { base, size } = entry {
                vec.push((*base, *size));
            }
        }
        vec
    }

    /// Get the first legacy interrupt number(IRQ).
    pub fn get_legacy_irq(&self) -> Option<u32> {
        for entry in self.0.iter().as_ref() {
            if let Resource::LegacyIrq(base) = entry {
                return Some(*base);
            }
        }
        None
    }

    /// Get information about the first PCI MSI interrupt resource.
    pub fn get_pci_msi_irqs(&self) -> Option<(u32, u32)> {
        self.get_msi_irqs(MsiIrqType::PciMsi)
    }

    /// Get information about the first PCI MSIx interrupt resource.
    pub fn get_pci_msix_irqs(&self) -> Option<(u32, u32)> {
        self.get_msi_irqs(MsiIrqType::PciMsix)
    }

    /// Get information about the first Generic MSI interrupt resource.
    pub fn get_generic_msi_irqs(&self) -> Option<(u32, u32)> {
        self.get_msi_irqs(MsiIrqType::GenericMsi)
    }

    fn get_msi_irqs(&self, ty: MsiIrqType) -> Option<(u32, u32)> {
        for entry in self.0.iter().as_ref() {
            if let Resource::MsiIrq {
                ty: msi_type,
                base,
                size,
            } = entry
            {
                if ty == *msi_type {
                    return Some((*base, *size));
                }
            }
        }
        None
    }

    /// Get the KVM memory slots to map memory into the guest.
    pub fn get_kvm_mem_slots(&self) -> Vec<u32> {
        let mut vec = Vec::new();
        for entry in self.0.iter().as_ref() {
            if let Resource::KvmMemSlot(index) = entry {
                vec.push(*index);
            }
        }
        vec
    }

    /// Get the first resource information for NIC MAC address.
    pub fn get_mac_address(&self) -> Option<String> {
        for entry in self.0.iter().as_ref() {
            if let Resource::MacAddresss(addr) = entry {
                return Some(addr.clone());
            }
        }
        None
    }

    /// Get immutable reference to all the resources.
    pub fn get_all_resources(&self) -> &[Resource] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIO_ADDRESS_SIZE: u16 = 5;
    const PIO_ADDRESS_BASE: u16 = 0;
    const MMIO_ADDRESS_SIZE: u64 = 0x8765_4321;
    const MMIO_ADDRESS_BASE: u64 = 0x1234_5678;
    const LEGACY_IRQ: u32 = 0x168;
    const PCI_MSI_IRQ_SIZE: u32 = 0x8888;
    const PCI_MSI_IRQ_BASE: u32 = 0x6666;
    const PCI_MSIX_IRQ_SIZE: u32 = 0x16666;
    const PCI_MSIX_IRQ_BASE: u32 = 0x8888;
    const GENERIC_MSI_IRQS_SIZE: u32 = 0x16888;
    const GENERIC_MSI_IRQS_BASE: u32 = 0x16688;
    const MAC_ADDRESS: &str = "00:08:63:66:86:88";
    const KVM_SLOT_ID: u32 = 0x0100;

    fn get_device_resource() -> DeviceResources {
        let entry = Resource::PioAddressRange {
            base: PIO_ADDRESS_BASE,
            size: PIO_ADDRESS_SIZE,
        };
        let mut resource = DeviceResources::new();
        resource.append(entry);
        let entry = Resource::MmioAddressRange {
            base: MMIO_ADDRESS_BASE,
            size: MMIO_ADDRESS_SIZE,
        };
        resource.append(entry);
        let entry = Resource::LegacyIrq(LEGACY_IRQ);
        resource.append(entry);
        let entry = Resource::MsiIrq {
            ty: MsiIrqType::PciMsi,
            base: PCI_MSI_IRQ_BASE,
            size: PCI_MSI_IRQ_SIZE,
        };
        resource.append(entry);
        let entry = Resource::MsiIrq {
            ty: MsiIrqType::PciMsix,
            base: PCI_MSIX_IRQ_BASE,
            size: PCI_MSIX_IRQ_SIZE,
        };
        resource.append(entry);
        let entry = Resource::MsiIrq {
            ty: MsiIrqType::GenericMsi,
            base: GENERIC_MSI_IRQS_BASE,
            size: GENERIC_MSI_IRQS_SIZE,
        };
        resource.append(entry);
        let entry = Resource::MacAddresss(MAC_ADDRESS.to_string());
        resource.append(entry);

        resource.append(Resource::KvmMemSlot(KVM_SLOT_ID));

        resource
    }

    #[test]
    fn get_pio_address_ranges() {
        let resources = get_device_resource();
        assert!(
            resources.get_pio_address_ranges()[0].0 == PIO_ADDRESS_BASE
                && resources.get_pio_address_ranges()[0].1 == PIO_ADDRESS_SIZE
        );
    }

    #[test]
    fn test_get_mmio_address_ranges() {
        let resources = get_device_resource();
        assert!(
            resources.get_mmio_address_ranges()[0].0 == MMIO_ADDRESS_BASE
                && resources.get_mmio_address_ranges()[0].1 == MMIO_ADDRESS_SIZE
        );
    }

    #[test]
    fn test_get_legacy_irq() {
        let resources = get_device_resource();
        assert!(resources.get_legacy_irq().unwrap() == LEGACY_IRQ);
    }

    #[test]
    fn test_get_pci_msi_irqs() {
        let resources = get_device_resource();
        assert!(
            resources.get_pci_msi_irqs().unwrap().0 == PCI_MSI_IRQ_BASE
                && resources.get_pci_msi_irqs().unwrap().1 == PCI_MSI_IRQ_SIZE
        );
    }

    #[test]
    fn test_pci_msix_irqs() {
        let resources = get_device_resource();
        assert!(
            resources.get_pci_msix_irqs().unwrap().0 == PCI_MSIX_IRQ_BASE
                && resources.get_pci_msix_irqs().unwrap().1 == PCI_MSIX_IRQ_SIZE
        );
    }

    #[test]
    fn test_get_generic_msi_irqs() {
        let resources = get_device_resource();
        assert!(
            resources.get_generic_msi_irqs().unwrap().0 == GENERIC_MSI_IRQS_BASE
                && resources.get_generic_msi_irqs().unwrap().1 == GENERIC_MSI_IRQS_SIZE
        );
    }

    #[test]
    fn test_get_mac_address() {
        let resources = get_device_resource();
        assert_eq!(resources.get_mac_address().unwrap(), MAC_ADDRESS);
    }

    #[test]
    fn test_get_kvm_slot() {
        let resources = get_device_resource();
        assert_eq!(resources.get_kvm_mem_slots(), vec![KVM_SLOT_ID]);
    }

    #[test]
    fn test_get_all_resources() {
        let resources = get_device_resource();
        assert_eq!(resources.get_all_resources().len(), 8);
    }
}
