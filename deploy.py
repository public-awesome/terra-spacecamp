# deploy to LocalTerra
from terra_sdk.client.lcd import LCDClient
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.util.contract import get_code_id, get_contract_address, read_file_as_b64
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
from terra_sdk.core.auth import StdFee
#from terra_sdk.client.lcd

terra = LCDClient(url='https://bombay-lcd.terra.dev', chain_id='bombay-10')

# creating the address (with a specific key)
deployer_key = MnemonicKey(mnemonic="dentist tray vital sudden noodle basic myself transfer margin sing ozone equal used chicken foil witness boil firm upgrade donor error there frost matrix")
acc2_key = MnemonicKey(mnemonic="brief session wing problem siege audit figure write firm road sword choose liberty alcohol card before unfold health daring income task radar rally once")

# creating the wallet objects to interact with the address
deployer = terra.wallet(deployer_key)
acc2 = terra.wallet(acc2_key)

print("deployer:", deployer.key.acc_address)
print("deployer balance:", terra.bank.balance(deployer.key.acc_address))

def store_contract(contract_name):
    contract_bytes = read_file_as_b64(f"artifacts/{contract_name}.wasm")
    store_code = MsgStoreCode(
        deployer.key.acc_address,
        contract_bytes
    )
    tx = deployer.create_and_sign_tx(
        msgs=[store_code], fee=StdFee(5000000, "1000000uluna")
    )
    result = terra.tx.broadcast(tx)
    code_id = get_code_id(result)
    return code_id

def instantiate_contract(code_id, init_msg):
    msg = MsgInstantiateContract(
        deployer.key.acc_address, 
        deployer.key.acc_address, 
        code_id,
        init_msg,
    )
 
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=StdFee(5000000, "1000000uluna")
    )
    result = terra.tx.broadcast(tx)
    contract_address = get_contract_address(result)
    return contract_address

def execute_contract(sender, contract_addr: str, execute_msg):
    msg = MsgExecuteContract(
        sender=sender.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg,
    )
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=StdFee(5000000, "1000000uluna")
    )
    result = terra.tx.broadcast(tx)
    return result

code_id_ts = store_contract("media")
print("code id:", code_id_ts)

inited_contract = instantiate_contract(code_id_ts, {
    "name": "New Coin",
    "symbol": "NC",
    "minter": deployer.key.acc_address,
})

print("contract id:", inited_contract)

# MINTING
minted_contract = execute_contract(
    deployer,
    inited_contract,
    {"mint":
        {
            "ask_amount": {"amount": "10", "denom": "uluna"},
            "base": 
                {
                    "token_id": "uniqueid1",
                    "owner": deployer.key.acc_address,
                    "name": "NFT",
                    "description": "the description",
                    "image": "https://picsum.photos/id/1025/200/300" 
                }
            
        }
    }
)
print("executed hash:", minted_contract.txhash)
print("owner of NFT:", terra.wasm.contract_query(inited_contract, {"owner_of": {"token_id":"uniqueid1"}}))
print("current ask of NFT:", terra.wasm.contract_query(inited_contract, {"current_ask_for_token": {"token_id":"uniqueid1"}})['ask'])

# BIDDING
new_bid = execute_contract(deployer, inited_contract, {
    "set_bid":
        {
            # 5 is <10, so the NFT won't transfer -- try >10 for NFT transfer
            "amount": {"amount": "5", "denom": "uluna"},
            "bidder": acc2.key.acc_address,
            "token_id": "uniqueid1"
        }
})
print("bid hash:", new_bid.txhash)
print("deployer:", deployer.key.acc_address)
print("deployer balance:", terra.bank.balance(deployer.key.acc_address))
print("acc2:", acc2.key.acc_address)
print("acc2 balance:", terra.bank.balance(acc2.key.acc_address))

print("owner of NFT:", terra.wasm.contract_query(inited_contract, {"owner_of": {"token_id":"uniqueid1"}}))