#![cfg_attr(not(feature = "std"), no_std)]

#[allow(dead_code)]
#[cfg(feature = "autopay")]
mod autopay;

#[cfg(feature = "flex")]
#[allow(dead_code)]
mod flex;

#[cfg(feature = "governance")]
#[allow(dead_code)]
mod governance;
