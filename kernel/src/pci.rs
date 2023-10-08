// TODO: Get this code to work to replace `tinypci`
//
// use alloc::{vec, vec::Vec};
// use anyhow::{bail, Result};
// use bitfield::{bitfield_bitrange, bitfield_debug, bitfield_fields};
// use core::{fmt, fmt::Formatter, mem};
// use num_derive::{FromPrimitive, ToPrimitive};
// use num_traits::{AsPrimitive, FromPrimitive, ToPrimitive};
// use x86_64::instructions::port::{Port, PortWriteOnly};
//
// #[derive(Copy, Clone, FromPrimitive, ToPrimitive)]
// pub struct Address(u32);
//
// bitfield_bitrange! { struct Address(u32) }
//
// impl Address {
//     bitfield_fields! {
//         u8;
//
//         bus, set_bus: 23, 16;
//         device, set_device: 15, 11;
//         func, set_func: 10, 8;
//         offset, set_offset: 7, 0;
//     }
//
//     pub fn new(device: u8, bus: u8) -> Self { Self::with_func_and_offset(bus,
// device, 0, 0) }
//
//     pub(self) fn with_func_and_offset(bus: u8, device: u8, func: u8, offset:
// u8) -> Self {         let mut address = Self(0x80000000);
//
//         address.set_bus(bus);
//         address.set_device(device);
//         address.set_func(func);
//         address.set_offset(offset & 0xfc);
//
//         address
//     }
//
//     pub fn ids(self) -> Result<(u16, u16)> {
//         let mut address = self;
//         address.set_offset(0);
//         let value: u32 = unsafe { config_read(address) };
//         let device_id = (value >> 16) as u16;
//         let vendor_id = value as u16;
//
//         if vendor_id == 0xff {
//             bail!("device does not exist");
//         }
//
//         Ok((device_id, vendor_id))
//     }
//
//     pub fn class(self) -> Class {
//         let mut address = self;
//         address.set_offset(0x8);
//         Class::from_u16(unsafe { config_read(address) }).unwrap()
//     }
//
//     pub fn header_type(self) -> u8 {
//         let mut address = self;
//         address.set_offset(0xc);
//         unsafe { config_read(address) }
//     }
// }
//
// pub struct DeviceInfo {
//     bus: u8,
//     device: u8,
//     device_id: u16,
//     vendor_id: u16,
//     class: Class,
//     header_type: u8,
// }
//
// impl DeviceInfo {
//     fn vendor_string(&self) -> &'static str {
//         match self.vendor_id {
//             0x8086 => "Intel Corp.",
//             _ => "Unknown",
//         }
//     }
// }
//
// impl fmt::Debug for DeviceInfo {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         let mut debug_struct = f.debug_struct(stringify!(DeviceInfo));
//         debug_struct.field("bus", &self.bus);
//         debug_struct.field("device", &self.device);
//         debug_struct.field("device_id", &self.device_id);
//         debug_struct.field("vendor", &format_args!("{:x}", &self.vendor_id));
//         debug_struct.field("class", &self.class);
//         debug_struct.field("header_type", &self.header_type);
//         debug_struct.finish()
//     }
// }
//
// #[non_exhaustive]
// #[repr(u8)]
// #[derive(Debug, FromPrimitive)]
// enum ClassCode {
//     MassStorage = 1,
//     Network,
//     Unknown = 255,
// }
//
// #[non_exhaustive]
// #[repr(u8)]
// #[derive(FromPrimitive)]
// enum MassStorage {
//     Ide = 0x1,
//     Ata = 0x5,
//     Sata = 0x6,
// }
//
// #[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
// pub struct Class(u16);
//
// bitfield_bitrange! { struct Class(u16) }
//
// impl Class {
//     bitfield_fields! {
//         u8;
//
//         _class_code , set_class_code: 15, 8;
//         subclass, set_subclass: 7, 0;
//     }
//
//     fn new(class_code: ClassCode, subclass: u8) -> Self {
//         let mut class = Self(0);
//         class.set_class_code(class_code as u8);
//         class.set_subclass(subclass);
//         class
//     }
//
//     pub fn class_code(self) -> ClassCode {
//         ClassCode::from_u8(self._class_code()).unwrap_or(ClassCode::Unknown)
//     }
// }
//
// impl fmt::Debug for Class {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         let mut debug_struct = f.debug_struct(stringify!(Class));
//         debug_struct.field("class_code", &self.class_code());
//         debug_struct.field("subclass", &self.subclass());
//         debug_struct.finish()
//     }
// }
//
// impl DeviceInfo {
//     fn new(bus: u8, device: u8) -> Result<DeviceInfo> {
//         let address = Address::new(bus, device);
//         let (device_id, vendor_id) = address.ids()?;
//
//         log::debug!("vendor_id: {vendor_id:x}");
//
//         Ok(DeviceInfo {
//             bus,
//             device,
//             device_id,
//             vendor_id,
//             class: address.class(),
//             header_type: address.header_type(),
//         })
//     }
// }
//
// trait ConfigValue: Copy + 'static
// where
//     u32: AsPrimitive<Self>,
// {
// }
//
// impl ConfigValue for u8 {}
// impl ConfigValue for u16 {}
// impl ConfigValue for u32 {}
//
// const ADDRESS_PORT: PortWriteOnly<u32> = PortWriteOnly::new(0xcf8);
// const DATA_PORT: Port<u32> = Port::new(0xcfc);
//
// unsafe fn config_read<T>(address: Address) -> T
// where
//     T: ConfigValue + Copy + 'static,
//     u32: AsPrimitive<T>,
// {
//     assert!(address.device() < 32);
//     assert!(address.func() < 8);
//
//     unsafe { ADDRESS_PORT.write(address.to_u32().unwrap()); }
//     let read = unsafe { DATA_PORT.read() };
//
//     let offset_bit = 4 - mem::size_of::<T>();
//
//     (read >> ((address.offset() & offset_bit as u8) * 8)) /*& 0xff)*/.as_()
// }
//
// pub fn brute_force_scan() -> Vec<DeviceInfo> {
//     let mut infos = vec![];
//     for bus in 0..=255 {
//         for device in 0..32 {
//             if let Ok(info) = DeviceInfo::new(bus, device) {
//                 infos.push(info);
//             }
//         }
//     }
//     infos
// }
