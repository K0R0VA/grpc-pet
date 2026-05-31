pub mod tracking {
    tonic::include_proto!("tracking");
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        tonic::include_file_descriptor_set!("tracking_descriptor");
}

pub mod auth {
    tonic::include_proto!("auth");
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        tonic::include_file_descriptor_set!("auth_descriptor");
}

pub mod price_receiver {
    tonic::include_proto!("price_receiver");
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        tonic::include_file_descriptor_set!("price_receiver_descriptor");
}