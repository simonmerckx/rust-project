//! File system with inode support + read and write operations on inodes
//!
//! Create a filesystem that has a notion of inodes and blocks, by implementing the [`FileSysSupport`], the [`BlockSupport`] and the [`InodeSupport`] traits together (again, all earlier traits are supertraits of the later ones).
//! Additionally, implement the [`InodeRWSupport`] trait to provide operations to read from and write to inodes
//!
//! [`FileSysSupport`]: ../../cplfs_api/fs/trait.FileSysSupport.html
//! [`BlockSupport`]: ../../cplfs_api/fs/trait.BlockSupport.html
//! [`InodeSupport`]: ../../cplfs_api/fs/trait.InodeSupport.html
//! [`InodeRWSupport`]: ../../cplfs_api/fs/trait.InodeRWSupport.html
//! Make sure this file does not contain any unaddressed `TODO`s anymore when you hand it in.
//!
//! # Status
//!
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! ...
//!

use thiserror::Error;
use cplfs_api::{controller::Device, error_given::{self, APIError}, fs::{BlockSupport, FileSysSupport, InodeRWSupport, InodeSupport}, types::{Block, Buffer, Inode, SuperBlock}};

use crate::b_inode_support::{self, CustomInodeFileSystem};

/// You are free to choose the name for your file system. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your file system data type so we can just use `FSName` instead of
/// having to manually figure out the name.
/// **TODO**: replace the below type by the type of your file system
pub type FSName = CustomInodeRWFileSystem;

// Custom type
/// Custom file system data type
pub struct CustomInodeRWFileSystem {
    inode_fs: CustomInodeFileSystem,
}

impl CustomInodeRWFileSystem {

    /// Create a new InodeCustomFileSystem given a BlockCustomFileSystem
    pub fn new(inodefs: CustomInodeFileSystem) -> CustomInodeRWFileSystem {
        CustomInodeRWFileSystem {  inode_fs: inodefs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in CustomInodeRWFileSystem
pub enum CustomInodeRWFileSystemError {
    /// An error occured in the inode layer
    #[error("InodeFileSystemError")]
    GivenError(#[from] b_inode_support::CustomInodeFileSystemError),
    /// The provided index is larger than the size of the file
    #[error("The provided index is larger than the size of the file")]
    IndexOutOfBounds,
    #[error("API error")]
    /// The input provided to some method in the controller layer was invalid
    APIError(#[from] error_given::APIError),
    #[error("The provided buffer is too small for the amount of bites that have to be written")]
    /// The provided buffer is too small for the amount of bytes that have to be written
    BufTooSmall,
    #[error("Writing the contents of the buffer at the given offset would make the inode exceed it's maximum size")]
    /// Writing the contents of the provided buffer starting at 
    /// the given offset would make the inode exceed it's maximum size
    WriteTooLarge

}


impl FileSysSupport for CustomInodeRWFileSystem {
    type Error = CustomInodeRWFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        return CustomInodeFileSystem::sb_valid(sb);
    }
    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        let inode_fs = CustomInodeFileSystem::mkfs(path, sb)?;
        return Ok(CustomInodeRWFileSystem::new(inode_fs))
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        let inode_fs = CustomInodeFileSystem::mountfs(dev)?;
        return Ok(CustomInodeRWFileSystem::new(inode_fs));
    }

    fn unmountfs(self) -> Device {
        return self.inode_fs.unmountfs();
    }
}

impl BlockSupport for CustomInodeRWFileSystem {
    fn b_get(&self, i: u64) -> Result<Block, Self::Error> {
        let block = self.inode_fs.b_get(i)?;
        return Ok(block);      
    }

    fn b_put(&mut self, b: &Block) -> Result<(), Self::Error> {
        let result = self.inode_fs.b_put(b)?;
        return Ok(result);
    }

    fn b_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let res = self.inode_fs.b_free(i)?;
        return Ok(res)       
    }

    fn b_zero(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.inode_fs.b_zero(i)?;
        return Ok(result);
    }

    fn b_alloc(&mut self) -> Result<u64, Self::Error> {
        let index = self.inode_fs.b_alloc()?;
        return Ok(index);
    }

    fn sup_get(&self) -> Result<SuperBlock, Self::Error> {
        let superblock = self.inode_fs.sup_get()?;
        return Ok(superblock);
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        let result = self.inode_fs.sup_put(sup)?;
        return Ok(result);
    }
}

impl InodeSupport for CustomInodeRWFileSystem {
    type Inode = Inode;

    fn i_get(&self, i: u64) -> Result<Self::Inode, Self::Error> {
        let inode = self.inode_fs.i_get(i)?;
        return Ok(inode);
    }

    fn i_put(&mut self, ino: &Self::Inode) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_put(ino)?;
        return Ok(result);
    }

    fn i_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_free(i)?;
        return Ok(result);
    }

    fn i_alloc(&mut self, ft: cplfs_api::types::FType) -> Result<u64, Self::Error> {
        let i = self.inode_fs.i_alloc(ft)?;
        return Ok(i);
    }

    fn i_trunc(&mut self, inode: &mut Self::Inode) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_trunc(inode)?;
        return Ok(result);
    }
}

impl InodeRWSupport for CustomInodeRWFileSystem {
    fn i_read(&self, inode: &Self::Inode, buf: &mut Buffer, off: u64, n: u64) -> Result<u64, Self::Error> {
        // If a read starts at inode.get_size(), returns with 0 bytes read.
        if off == inode.disk_node.size {
            return Ok(0);
        }
        // returns an error and does not read anything if index falls further outside of the file's bounds. 
        if off > inode.disk_node.size {
            return Err(CustomInodeRWFileSystemError::IndexOutOfBounds);
        }

        let superblock = self.sup_get()?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/superblock.block_size as f64).ceil(); 
        let mut buf_offset = 0;
        for index in 0..(nb_selected_blocks as u64) {
            // skip the blocks that don't contain bytes we need
            if (index +1)*superblock.block_size < off {
                continue
            }
            // we only want to read n bytes, also stop if buf is full
            if buf_offset >= n || buf_offset >= buf.len() {
                break
            }
            let element = file_blocks[index as usize];
            if !(element == 0) {
                // b-get: read the nth block of the entire disk and return it
                let block = self.b_get(element)?;
                //let mut offset = 0;
                for byte_index in 0..(superblock.block_size) {
                    // we only want to read n bytes and stop when end of file is reached
                    if buf_offset >= n || buf_offset >= inode.disk_node.size {
                        break
                    };
                    // start reading from byte offset off in the inode 
                    if index * superblock.block_size + byte_index >= off {
                        let mut byte: [u8;1] = [0];
                        block.read_data(&mut byte, byte_index)?;
                        // If buf cannot hold n bytes of data, reads until buf is full instead.
                        match buf.write_data(&byte, buf_offset) {
                            // reached end of the buf stop adding
                            Err(APIError::BlockInput("Trying to write beyond the bounds of the block",)) => break,
                            // not specified what to do in other cases
                            Err(_) => (),
                            Ok(_) => ()
                        }
                        buf_offset += 1;
                    }               
                }
            }
        }
        return Ok(buf_offset);
    }

    fn i_write(&mut self,inode: &mut Self::Inode,buf: &cplfs_api::types::Buffer,off: u64, n: u64) -> Result<(), Self::Error> {
        // returns an error and does not read anything if index falls further outside of the file's bounds. 
        if off > inode.disk_node.size {
            return Err(CustomInodeRWFileSystemError::IndexOutOfBounds);
        }

        // Returns an error if buf cannot hold at least n bytes of data.
        if buf.len() < n {
            return Err(CustomInodeRWFileSystemError::BufTooSmall);
        }

        // If the write would make the inode exceed its maximum possible size, do nothing and return an error.
        let sb = self.sup_get()?;
        if off + n > inode.disk_node.direct_blocks.len() as u64 * sb.block_size {
            return Err(CustomInodeRWFileSystemError::WriteTooLarge);
        }

        // Check if the provided inode is large enough, otherwise extend it 
        // if necessary, start allocating extra blocks to expand the file and continue writing into the new blocks.
        let current_amount_blocks = (inode.disk_node.size as f64/sb.block_size as f64).ceil();
        if off + n > (current_amount_blocks as u64 * sb.block_size) {
            let remaining_bytes = (off + n) - inode.disk_node.size;
            let amount_of_new_blocks = (remaining_bytes as f64 / sb.block_size as f64).ceil();
            for i in 0..amount_of_new_blocks as u64 {
                let new_block_index = sb.datastart + self.b_alloc()?;
                inode.disk_node.direct_blocks[(current_amount_blocks + i as f64) as usize] = new_block_index;
            }
            inode.disk_node.size = off + n;
            self.i_put(inode)?;
        }

        // if we have enough blocks but they are not all fully used yet
        // this if is only entered when we already have a partly
        // unused block assinged to an inode
        if off + n <  (current_amount_blocks as u64 * sb.block_size) && (off + n) > inode.disk_node.size { 
            inode.disk_node.size  = off + n;
        }

        // write changes back
        self.i_put(inode)?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/sb.block_size as f64).ceil(); 
        let mut buf_offset = 0;
        for index in 0..(nb_selected_blocks as u64) {
            // skip the blocks that don't contain bytes we need
            if (index +1)*sb.block_size < off {
                continue
            }
            // we only want to read n bytes, also stop if buf is full
            if buf_offset >= n {
                break
            }
            let element = file_blocks[index as usize];
            if !(element == 0) {
                // b-get: read the nth block of the entire disk and return it
                let mut block = self.b_get(element)?;
                for byte_index in 0..(sb.block_size)  {
                    if buf_offset >= n  {
                        break
                    };
                    // write only if we are over offset
                    if index * sb.block_size + byte_index >= off {
                        let mut byte: [u8;1] = [0];
                        // read the info out of the buffer into a byte
                        buf.read_data(&mut byte, buf_offset)?;
                        // write the byte into the inode
                        match block.write_data(&byte, byte_index) {
                            // reached end of the buf, so stop adding
                            Err(APIError::BlockInput("Trying to write beyond the bounds of the block",)) => break,
                            // not specified what to do in other cases
                            Err(_) => (),
                            Ok(_) => ()
                        }
                        buf_offset += 1;
                    }
                    self.b_put(&block)?;
                }
            }
        }
        return Ok(())
    }
}


// **TODO** define your own tests here.

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "e", feature = "all")))]
#[path = "../../api/fs-tests/e_test.rs"]
mod tests;
