use ethers::{
    abi::{encode, Token},
    core::types::{Address, U256, Bytes, TransactionRequest},
    providers::{Http, Provider, Middleware},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
    types::transaction::eip2718::TypedTransaction,
};
use std::{
    io::{self, Write},
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn function_selector(signature: &str) -> [u8; 4] {
    use ethers::utils::keccak256;
    let hash = keccak256(signature.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

async fn call_contract<M: Middleware>(
    provider: &M,
    to: Address,
    data: Bytes,
) -> Result<Bytes> {
    let tx: TypedTransaction = TransactionRequest::new().to(to).data(data).into();
    let output = provider.call(&tx, None).await.map_err(|e| e.to_string())?;
    Ok(output)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Input Url RPC/Node Blockchain Network: ");
    let mut rpc_url = String::new();
    io::stdin().read_line(&mut rpc_url)?;
    let rpc_url = rpc_url.trim();

    println!("Input Chain ID Blockchain Network: ");
    let mut chain_id_str = String::new();
    io::stdin().read_line(&mut chain_id_str)?;
    let chain_id: u64 = chain_id_str.trim().parse()?;

    let provider = Provider::<Http>::try_from(rpc_url)?
        .interval(std::time::Duration::from_millis(100));
    let provider = Arc::new(provider);

    if provider.get_block_number().await.is_err() {
        println!("Error connecting to RPC");
        return Ok(());
    }
    println!("Web3 Connected...\n");

    println!("Enter Your Address 0x...: ");
    let mut sender_str = String::new();
    io::stdin().read_line(&mut sender_str)?;
    let sender = Address::from_str(sender_str.trim())?;

    println!("Enter Your Private Key (no 0x): ");
    let mut pk = String::new();
    io::stdin().read_line(&mut pk)?;
    let wallet = LocalWallet::from_str(pk.trim())?.with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let client = Arc::new(client);

    println!("Enter Token Address 0x...: ");
    let mut token_addr_str = String::new();
    io::stdin().read_line(&mut token_addr_str)?;
    let token_addr = Address::from_str(token_addr_str.trim())?;

    println!("Enter Contract Address Router DEX: ");
    let mut router_addr_str = String::new();
    io::stdin().read_line(&mut router_addr_str)?;
    let router_addr = Address::from_str(router_addr_str.trim())?;

    // --- Get WETH address ---
    println!("Fetching WETH address...");
    let weth_selector = function_selector("WETH9()");
    let weth_output = call_contract(&provider, router_addr, Bytes::from(weth_selector.to_vec())).await?;
    if weth_output.len() < 32 {
        return Err("Invalid WETH9 response".into());
    }
    let weth_addr = Address::from_slice(&weth_output[weth_output.len() - 20..]);

    // --- Get token info ---
    let name_selector = function_selector("name()");
    let symbol_selector = function_selector("symbol()");
    let decimals_selector = function_selector("decimals()");

    let call_name = call_contract(&provider, token_addr, Bytes::from(name_selector.to_vec())).await?;
    let call_symbol = call_contract(&provider, token_addr, Bytes::from(symbol_selector.to_vec())).await?;
    let call_decimals = call_contract(&provider, token_addr, Bytes::from(decimals_selector.to_vec())).await?;

    let token_name = decode_string(&call_name)?;
    let token_symbol = decode_string(&call_symbol)?;
    let decimals = if !call_decimals.is_empty() {
        call_decimals[call_decimals.len() - 1] as u8
    } else {
        18
    };

    // --- Balances ---
    let eth_balance = provider.get_balance(sender, None).await?;
    let eth_balance_eth = eth_balance / U256::from(10u128.pow(18));
    println!("Your Balance: {} ETH/BNB", eth_balance_eth);

    let balance_selector = function_selector("balanceOf(address)");
    let balance_data = encode(&[Token::Address(sender)]);
    let balance_call = [&balance_selector[..], &balance_data].concat();
    let token_balance_raw = call_contract(&provider, token_addr, Bytes::from(balance_call)).await?;
    let token_balance = U256::from_big_endian(&token_balance_raw);
    let token_balance_human = token_balance / U256::from(10u128.pow(decimals as u32));
    println!("Token Balance: {} {}", token_balance_human, token_symbol);


    // --- Input amount ---
    println!("\nEnter Amount You Want To Buy [ETH/BNB]: ");
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount_f64: f64 = amount_str.trim().parse()?;
    let amount_wei = U256::from((amount_f64 * 1e18) as u128);

    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 1_000_000;

    println!("\nSkipping WETH approve (using native ETH/BNB directly)...");

    // --- exactInputSingle ---
    println!("\nProcessing Swap Buy: {} ETH for {}", amount_f64, token_name);

    let params = vec![
        Token::Address(weth_addr),
        Token::Address(token_addr),
        Token::Uint(U256::from(500u32)),
        Token::Address(sender),
        Token::Uint(amount_wei),
        Token::Uint(U256::zero()),
        Token::Uint(U256::zero()),
        Token::Uint(U256::from(deadline)),
    ];

    let encoded_params = encode(&[Token::Tuple(params)]);
    let sig = "exactInputSingle((address,address,uint24,address,uint256,uint256,uint160,uint256))";
    let selector = function_selector(sig);
    let calldata = [&selector[..], &encoded_params].concat();

    let tx: TypedTransaction = TransactionRequest::new()
        .to(router_addr)
        .value(amount_wei)
        .data(Bytes::from(calldata))
        .from(sender)
        .into();

    let pending_tx = client.send_transaction(tx, None).await?;
    let tx_hash = *pending_tx;
    println!("\n✅ Swap Transaction Sent!");
    println!("Tx Hash: 0x{:x}", tx_hash);

    Ok(())
}

fn decode_string(data: &[u8]) -> Result<String> {
    if data.len() < 32 {
        return Ok(String::new());
    }
    let len_bytes: [u8; 32] = data[..32].try_into().map_err(|_| "Invalid length")?;
    let len = U256::from_big_endian(&len_bytes).as_usize();
    let start = 32;
    let end = std::cmp::min(start + len, data.len());
    let s = String::from_utf8_lossy(&data[start..end]).to_string();
    Ok(s)
}
