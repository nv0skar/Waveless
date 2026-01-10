// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

pub mod project;

use rustyrosetta::*;

use std::{collections::HashMap, net::SocketAddr};

use compact_str::*;
use derive_more::{Constructor, Display};
use getset::*;
use serde::{Deserialize, Serialize};

pub type DatabaseId = CompactString;

pub type DataSchemaDiscoveryMethodId = CompactString;

pub type ExternalDriverId = CompactString;
