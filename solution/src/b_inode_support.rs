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
//! COMPLETED: PARTIAL
//!
//! COMMENTS:
//!
//! ...
//!
use a_block_support::CustomBlockFileSystemError;
// import SuperBlock
use cplfs_api::{fs::InodeSupport, types::{DInode, SuperBlock}};
// import BlockSupport
use cplfs_api::fs::BlockSupport;
use cplfs_api::types::{Block, Inode};
use cplfs_api::{controller::Device, error_given, fs::FileSysSupport, types::FType, types::{DINODE_SIZE, SUPERBLOCK_SIZE}};
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
    block_system: CustomBlockFileSystem,
    inode_start: u64,
    nb_inodes_block: u64
}

impl CustomInodeFileSystem {

    /// Create a new InodeCustomFileSystem given a BlockCustomFileSystem
    pub fn new(blockfs: CustomBlockFileSystem, is: u64, nib: u64) -> CustomInodeFileSystem {
        CustomInodeFileSystem {  block_system: blockfs, inode_start: is, nb_inodes_block: nib }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in my implementation
pub enum CustomInodeFileSystemError {
    /// An error occured in the block layer
    #[error("BlockFileSystemError")]
    GivenError(#[from] a_block_support::CustomBlockFileSystemError),
    #[error("API error")]
    /// The input provided to some method in the controller layer was invalid
    APIError(#[from] error_given::APIError),
    #[error("The provided inode index is out of bounds")]
    /// Error thrown when an index is greater than the number of 
    /// inodes in the system.
    InodeIndexOutOfBounds,
    #[error("The inode trying to be freed is already free")]
    /// Error thrown when the inode that is trying
    /// to be freed is already free.
    InodeAlreadyFree,
    #[error("There is no free inode available")]
    /// Thrown when there is no free inode available
    NoFreeInode,
    #[error("The block that was tried to be freed is already free")]
    /// Thrown when the block that is trying to be freed is already free
    BlockIsAlreadyFree,


}


impl FileSysSupport for CustomInodeFileSystem {
    type Error = CustomInodeFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        return CustomBlockFileSystem::sb_valid(sb);
    }

    // watch out for the inodes; an all-0 inode will not necessarily come out well during deserialization, 
    // and probably needs to be overwritten by an actually free inode
    // if you need to read/write multiple inodes in the same block, only load and store this block once!
    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        let mut fs = CustomBlockFileSystem::mkfs(path, sb)?;
        let inodestart = sb.inodestart;
        let nb_inodes_block = sb.block_size / *DINODE_SIZE;
        let blocks = sb.bmapstart - inodestart;
        // for every inode block
        for x in 0..blocks{
            // The number of inodes does not 
            // necessarily have to fill up the entire region
            let block_stop = x * nb_inodes_block;
            
            if block_stop > sb.ninodes {
                break
            }
            let mut block = fs.device.read_block(inodestart + x)?;
            // for every inode in this in block
            for y in 0..nb_inodes_block {
                // The number of inodes does not 
                // necessarily have to fill up the entire region
                let stopcond2 = y + block_stop;
                if stopcond2 > sb.ninodes{
                    break
                }
                let dinode = DInode::default();
                let offset = y * (*DINODE_SIZE);
                block.serialize_into(&dinode, offset)?;
                fs.device.write_block(&block)?;
            }
            
        }
        return Ok(CustomInodeFileSystem::new(fs, inodestart, nb_inodes_block))
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        let block_fs = CustomBlockFileSystem::mountfs(dev)?;
        let nb_inodes_block = block_fs.superblock.block_size / *DINODE_SIZE;
        let inode_start = block_fs.superblock.inodestart;
        return Ok(CustomInodeFileSystem::new(block_fs,inode_start , nb_inodes_block));
    }

    fn unmountfs(self) -> Device {
        return self.block_system.device;
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
        let res = self.block_system.b_free(i)?;
        return Ok(res)
        //watch out; absolute indices
        /* 
        let res = match self.block_system.b_free(i) {
            // Custom error so we can check in i_free
            Err(CustomBlockFileSystemError::BlockIsAlreadyFree) => Err(CustomInodeFileSystemError::BlockIsAlreadyFree),
            Err(e) => Err(e)?,
            Ok(_) => Ok(())

        };
        println!("{:?}", res);
        */
        
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
        let superblock = self.block_system.sup_get()?;
        return Ok(superblock);
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        let result = self.block_system.sup_put(sup)?;
        return Ok(result);
    }
}


impl InodeSupport for CustomInodeFileSystem {
    type Inode = Inode;

    fn i_get(&self, i: u64) -> Result<Self::Inode, Self::Error> {
        if i > self.block_system.superblock.ninodes - 1{
            return Err(CustomInodeFileSystemError::InodeIndexOutOfBounds);
        }
        let required_block = i / self.nb_inodes_block;
        let block = self.block_system.device.read_block(self.inode_start + required_block)?;
        let offset = (i % self.nb_inodes_block) * (*DINODE_SIZE);
        let dinode = block.deserialize_from(offset)?;
        return Ok(Inode::new(i, dinode));
    }

    fn i_put(&mut self, ino: &Self::Inode) -> Result<(), Self::Error> {
        let block_nb = ino.inum / self.nb_inodes_block;
        let mut block = self.block_system.device.read_block(self.inode_start + block_nb)?;
        let offset = (ino.inum % self.nb_inodes_block) * (*DINODE_SIZE);
        block.serialize_into(&ino.disk_node, offset)?;
        let result = self.b_put(&block)?;
        return Ok(result);
    }

    fn i_free(&mut self, i: u64) -> Result<(), Self::Error> {
        if i > self.block_system.superblock.ninodes - 1  {
            return Err(CustomInodeFileSystemError::InodeIndexOutOfBounds);
        }

        let mut inode = self.i_get(i)?;    
        if inode.disk_node.ft == FType::TFree {
            return Err(CustomInodeFileSystemError::InodeAlreadyFree);
        }
        
        if inode.disk_node.nlink == 0 {
            let file_blocks = inode.disk_node.direct_blocks;
            for index in &file_blocks{
                // self.block_system.superblock.datastart < *index 
                // 0 elements in array
                if !(*index == 0) {
                    self.b_free(*index - self.block_system.superblock.datastart)?;
                }
                
                /* 
                let _ = match self.b_free(*i) {
                    // The block is actually freed so no problem
                    Err(CustomInodeFileSystemError::BlockIsAlreadyFree) => Ok(()),
                    // Other errors should be passed
                    Err(e) => Err(e),
                    Ok(_) => Ok(())
                };
                */
            }
            inode.disk_node.ft = FType::TFree;
            inode.disk_node.direct_blocks = [0,0,0,0,0,0,0,0,0,0,0,0 as u64];
            self.i_put(&inode)?;
        }
        return Ok(())
    }

    fn i_alloc(&mut self, ft: cplfs_api::types::FType) -> Result<u64, Self::Error> {
        let ninodes = self.block_system.superblock.ninodes;
        // The inode with index 0 should never be allocated.
        for y in 1..ninodes {
            let mut inode = self.i_get(y)?;
            if inode.disk_node.ft == FType::TFree {
                inode.disk_node.ft = ft;
                inode.disk_node.size = 0;
                inode.disk_node.nlink = 0;
                self.i_put(&inode)?;
                return Ok(y);
            }
        }      
        return Err(CustomInodeFileSystemError::NoFreeInode)
    }

    fn i_trunc(&mut self, inode: &mut Self::Inode) -> Result<(), Self::Error> {
        //let mut disk_inode = self.i_get(inode.inum)?;
        inode.disk_node.size = 0;
        let file_blocks = inode.disk_node.direct_blocks;
        for index in &file_blocks{
            if !(*index == 0) {
                self.b_free(*index - self.block_system.superblock.datastart)?;
            }
        }
        inode.disk_node.direct_blocks = [0,0,0,0,0,0,0,0,0,0,0,0 as u64];
        self.i_put(&inode)?;

        return Ok(())
    }
}


// **TODO** define your own tests here.

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "b", feature = "all")))]
#[path = "../../api/fs-tests/b_test.rs"]
mod tests;
