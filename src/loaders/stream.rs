#[cfg(feature = "debug")]
use std::cell::UnsafeCell;

use derive_new::new;

use super::convertable::ConversionError;
use super::version::InternalVersion;
use super::{Named, Version};
#[cfg(feature = "debug")]
use crate::debug::*;
#[cfg(feature = "debug")]
use crate::interface::PacketEntry;
#[cfg(feature = "debug")]
use crate::interface::TrackedState;
use crate::interface::WeakElementCell;
use crate::loaders::convertable::check_upper_bound;
#[cfg(feature = "debug")]
use crate::network::IncomingPacket;

#[derive(new)]
pub struct ByteStream<'b> {
    data: &'b [u8],
    #[new(default)]
    offset: usize,
    #[new(default)]
    version: Option<InternalVersion>,
    #[cfg(feature = "debug")]
    #[new(default)]
    packet_history: Vec<PacketEntry>,
}

impl<'b> ByteStream<'b> {
    pub fn next<S: Named>(&mut self) -> Result<u8, Box<ConversionError>> {
        check_upper_bound::<S>(self.offset, self.data.len())?;
        let byte = self.data[self.offset];
        self.offset += 1;
        Ok(byte)
    }

    pub fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }

    pub fn set_version<T>(&mut self, version: Version<T>) {
        self.version = Some(version.into());
    }

    pub fn get_version(&mut self) -> InternalVersion {
        self.version.unwrap()
    }

    pub fn slice<S: Named>(&mut self, count: usize) -> Result<&[u8], Box<ConversionError>> {
        check_upper_bound::<S>(self.offset + count, self.data.len() + 1)?;

        let start_index = self.offset;
        self.offset += count;

        Ok(&self.data[start_index..self.offset])
    }

    pub fn skip(&mut self, count: usize) {
        self.offset += count;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset
    }

    pub fn remaining_bytes(&mut self) -> Vec<u8> {
        let end_index = self.data.len();
        let data = self.data[self.offset..end_index].to_vec();
        self.offset = end_index;
        data
    }

    #[cfg(feature = "debug")]
    pub fn incoming_packet<T: IncomingPacket + Clone + 'static>(&mut self, packet: &T) {
        self.packet_history.push(PacketEntry::new_incoming(packet, T::NAME, T::IS_PING));
    }

    #[cfg(feature = "debug")]
    pub fn transfer_packet_history<const N: usize>(
        &mut self,
        packet_history: &mut TrackedState<RingBuffer<(PacketEntry, UnsafeCell<Option<WeakElementCell>>), N>>,
    ) {
        if !self.packet_history.is_empty() {
            packet_history.with_mut(|buffer, changed| {
                self.packet_history
                    .drain(..)
                    .for_each(|packet| buffer.push((packet, UnsafeCell::new(None))));
                changed()
            });
        }
    }

    #[cfg(feature = "debug")]
    pub fn assert_empty(&self, file_name: &str) {
        let remaining = self.data.len() - self.offset;

        if remaining != 0 {
            print_debug!(
                "incomplete read on file {}{}{}; {}{}{} bytes remaining",
                MAGENTA,
                file_name,
                NONE,
                YELLOW,
                remaining,
                NONE
            );
        }
    }
}
