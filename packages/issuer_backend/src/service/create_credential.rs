use crate::eth::create_signing_message;
use crate::guard::authenticated;
use crate::passport::get_passport_score;
use crate::{
    eth::{recover_eth_address, EthAddress, EthSignature},
    ETH_PRINCIPAL, PRINCIPAL_SCORE,
};
use ic_cdk::{caller, update};

#[update(guard = authenticated)]
pub async fn create_credential(signature: String, address: String) -> Result<f32, String> {
    let caller_principal: [u8; 29] = caller().as_slice()[..29]
        .try_into()
        .map_err(|_| "Invalid principal".to_string())?;

    // Function can only be called once per principal.
    PRINCIPAL_SCORE.with_borrow(|s| {
        if s.contains_key(&caller_principal) {
            return Err("Principal already registered".to_string());
        }
        Ok(())
    })?;

    // Create an EthAddress from the string. This validates the address.
    let address = EthAddress::new(&address)?;

    // Function can only be called once per address.
    ETH_PRINCIPAL.with_borrow(|e| {
        if e.contains_key(&address.as_hash()) {
            return Err("Address already registered".to_string());
        }
        Ok(())
    })?;

    // Create an EthSignature from the string. This validates the signature.
    let signature = EthSignature::new(&signature)?;

    // Create a message string to recover the address from the signature.
    let message = create_signing_message(&address, &caller());

    // Compare the address recovered from the signature with the address provided.
    let recovered_address = recover_eth_address(&message, &signature)?;
    if recovered_address != address.as_str() {
        return Err("Invalid signature".to_string());
    }

    let score = get_passport_score(&address).await?;

    ETH_PRINCIPAL.with_borrow_mut(|e| {
        e.insert(address.as_hash(), caller_principal);
    });

    PRINCIPAL_SCORE.with_borrow_mut(|s| {
        s.insert(caller_principal, score);
    });

    Ok(score)
}
