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
use cplfs_api::{fs::InodeSupport, types::{DInode, SuperBlock}};
// import BlockSupport
use cplfs_api::fs::BlockSupport;
use cplfs_api::types::{Block, Inode};
use cplfs_api::{controller::Device, error_given, fs::FileSysSupport, types::Buffer, types::{DINODE_SIZE, SUPERBLOCK_SIZE}};
use thiserror::Error;

use crate::a_block_support::{self, CustomBlockFileSystem};
/// You are free to choose the name for your file system. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your file system data type so we can just use `FSName` instead of
/// having to manually figure out the name.
/// **TODO**: replace the below type by the type of your file system
pub type FSName = CustomInodeFileSystem;

// Custom type
/// Custom file system data type
pub struct CustomInodeFileSystem {
    //device: Device,
    block_system: CustomBlockFileSystem
}

impl CustomInodeFileSystem {

    /// Create a new InodeCustomFileSystem given a BlockCustomFileSystem
    pub fn new(blockfs: CustomBlockFileSystem) -> CustomInodeFileSystem {
        CustomInodeFileSystem {  block_system: blockfs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in my implementation
pub enum CustomInodeFileSystemError {
    /// An error occured in the block layer
    #[error("BlockFileSystemError")]
    GivenError(#[from] a_block_support::CustomBlockFileSystemError)
}

impl FileSysSupport for CustomInodeFileSystem {
    type Error = CustomInodeFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        return CustomBlockFileSystem::sb_valid(sb);
    }

    // watch out for the inodes; an all-0 inode will not necessarily come out well during deserialization, and probably needs to be overwritten by an actually free inode
    // if you need to read/write multiple inodes in the same block, only load and store this block once!
    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        let fs = CustomBlockFileSystem::mkfs(path, sb)?;
        let inodestart = sb.inodestart;
        let nb_inodes_block = sb.block_size / *DINODE_SIZE;
        let blocks = sb.ninodes / nb_inodes_block;
        // for every inode block
        for x in 0..blocks{
            let mut block = fs.device.read_block(x)?;
            // for every inode in this in block
            for y in 0..nb_inodes_block {
                let inode = DInode::default();
                let offset = y * (*DINODE_SIZE);
                block.serialize_into(&inode, offset)?;
                fs.device.write_block(inode);
            }
            
        }


        return Ok(CustomInodeFileSystem::new(fs))
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        todo!()
    }

    fn unmountfs(self) -> Device {
        todo!()
    }
}

impl BlockSupport for CustomInodeFileSystem {
    fn b_get(&self, i: u64) -> Result<Block, Self::Error> {
        let block = self.block_system.b_get(i)?;
        return Ok(block);      
    }

    fn b_put(&mut self, b: &Block) -> Result<(), Self::Error> {
        let result = self.block_system.b_put(b)?;
        return Ok(result);
    }

    fn b_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.block_system.b_free(i)?;
        return Ok(result)
    }

    fn b_zero(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.block_system.b_zero(i)?;
        return Ok(result);
    }

    fn b_alloc(&mut self) -> Result<u64, Self::Error> {
        let index = self.block_system.b_alloc()?;
        return Ok(index);
    }

    fn sup_get(&self) -> Result<SuperBlock, Self::Error> {
        let superblock = self.sup_get()?;
        return Ok(superblock);
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        let result = self.sup_put(sup)?;
        return Ok(result);
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
