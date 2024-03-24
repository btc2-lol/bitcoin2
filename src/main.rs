#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hex: [u8; 8] = hex::decode("dfcec48bb8491856c353306ab5febeb7e99e4d783eedf3de98f3ee0812b92bad").unwrap()[..8].try_into().unwrap();
        // Reverse the iterator, clone the elements, and collect into a new Vec<u8>
        // let reversed_hex: Vec<u8> = hex.iter().rev().cloned().collect();

        // Now, take the first 4 elements from the reversed vector and collect them into a new Vec<u8>
        println!("{:?}", &hex);
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    //     .await
    //     .unwrap();
    // axum::serve(listener, bitcoin2::app()).await.unwrap();

    Ok(())
}

fn find_output(tx_id: [u8; 32], vout: u16) {
    const tx_id = iter().rev().cloned().collect()[0..4];
    get_chunk("utxos", |&chunk| chunk[0..10] == [
        tx_id,
        vout.to_be_bytes()
    ])
}

fn get_chunk<P, const N: usize>(file_path: &str) Option<[u8; N]> where
P: Fn(&[u8; N]) -> bool;
-> io::Result<()> {
    // Open the file
    let mut file = File::open(file_path)?;

    // Create a buffer with the specified chunk size
    let mut buffer = vec![0; ];

    // Loop to read the file chunk by chunk
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            // End of file reached
            break;
        }

        // Process the chunk here
        // For demonstration, we'll just print the size of the chunk read
        println!("Read a chunk of size: {}", bytes_read);

        // If you need to work with the exact data read, use &buffer[..bytes_read]
        // Do something with the chunk...
    }

    Ok(())
}

