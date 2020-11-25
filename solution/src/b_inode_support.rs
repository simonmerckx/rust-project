//! File system with inode support
//!
//! Create a filesystem that has a notion of inodes and blocks, by implementing the [`FileSysSupport`], the [`BlockSupport`] and the [`InodeSupport`] traits together (again, all earlier traits are supertraits of the later ones).
//!
//! [`FileSysSupport`]: ../../cplfs_api/fs/trait.FileSysSupport.html
//! [`BlockSupport`]: ../../cplfs_api/fs/trait.BlockSupport.html
//! [`InodeSupport`]: ../../cplfs_api/fs/trait.InodeSupport.html
//! Make sure this file does not contain any unaddressed `TODO`s anymore when you hand it in.
//!
//! # Status
//!
//! **TODO**: Replace the question mark below with YES, NO, or PARTIAL to
//! indicate the status of this assignment. If you want to tell something
//! about this assignment to the grader, e.g., you have a bug you can't fix,
//! or you want to explain your approach, write it down after the comments
//! section. If you had no major issues and everything works, there is no need to write any comments.
//!
//! COMPLETED: ?
//!
//! COMMENTS:
//!
//! ...
//!
// import SuperBlock
use cplfs_api::{fs::InodeSupport, types::SuperBlock};
// import BlockSupport
use cplfs_api::fs::BlockSupport;
use cplfs_api::types::{Block, Inode};
use cplfs_api::{controller::Device, error_given, fs::FileSysSupport, types::Buffer, types::{DINODE_SIZE, SUPERBLOCK_SIZE}};
use thiserror::Error;

use crate::a_block_support::{self, BlockCustomFileSystem};
/// You are free to choose the name for your file system. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your file system data type so we can just use `FSName` instead of
/// having to manually figure out the name.
/// **TODO**: replace the below type by the type of your file system
pub type FSName = InodeCustomFileSystem;

// Custom type
/// Custom file system data type
pub struct InodeCustomFileSystem {
    device: Device,
    block_system: BlockCustomFileSystem
}

impl InodeCustomFileSystem {

    /// Create a new CustomFileSystem given a Device dev
    pub fn new(dev: Device, fs: BlockCustomFileSystem) -> InodeCustomFileSystem {
        InodeCustomFileSystem { device: dev, block_system: fs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in my implementation
pub enum CustomFileSystemError {
    #[error("invalid SuperBlock was detected")]
    /// Error thrown when an invalid superblock is encountered
    InvalidSuperBlock,
    #[error("SuperBlock and device are not compatible")]
    /// Thrown when the device and superblock don't agree
    IncompatibleDeviceSuperBlock,
    #[error("The data index is out of bounds for this device")]
    /// Thrown when the block index provided is larger than ndatablocks - 1
    DataIndexOutOfBounds,
    #[error("The block that was tried to be freed is already free")]
    /// Thrown when the block that is trying to be freed is already free
    BlockIsAlreadyFree,
    #[error("There is no free data block")]
    /// Thrown when there is no free data block 
    NoFreeDataBlock,
    /// The input provided to some method in the controller layer was invalid
    #[error("API error")]
    GivenError(#[from] error_given::APIError)
}

impl FileSysSupport for InodeCustomFileSystem {
    type Error = CustomFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        todo!()
    }

    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        todo!()
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        todo!()
    }

    fn unmountfs(self) -> Device {
        todo!()
    }
}

impl BlockSupport for InodeCustomFileSystem {
    fn b_get(&self, i: u64) -> Result<Block, Self::Error> {
        todo!()
    }

    fn b_put(&mut self, b: &Block) -> Result<(), Self::Error> {
        todo!()
    }

    fn b_free(&mut self, i: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn b_zero(&mut self, i: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn b_alloc(&mut self) -> Result<u64, Self::Error> {
        todo!()
    }

    fn sup_get(&self) -> Result<SuperBlock, Self::Error> {
        todo!()
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        todo!()
    }
}

/*
impl InodeSupport for BlockCustomFileSystem {
    type Inode;

    fn i_get(&self, i: u64) -> Result<Self::Inode, Self::Error> {
        todo!()
    }

    fn i_put(&mut self, ino: &Self::Inode) -> Result<(), Self::Error> {
        todo!()
    }

    fn i_free(&mut self, i: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn i_alloc(&mut self, ft: cplfs_api::types::FType) -> Result<u64, Self::Error> {
        todo!()
    }

    fn i_trunc(&mut self, inode: &mut Self::Inode) -> Result<(), Self::Error> {
        todo!()
    }
}
*/

// **TODO** define your own tests here.

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "b", feature = "all")))]
#[path = "../../api/fs-tests/b_test.rs"]
mod tests;
