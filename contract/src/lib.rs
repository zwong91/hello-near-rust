// Anatomy 解剖一个智能合约
//任何支持编译目标 wasm32-unknown-unknown的东西， 都是和智能合约兼容的
// 对编译合约二进制文件有一个大小限制，约为4.19 MB
// Modules 为NEAR SDK，提供访问执行环境，允许调用其他合同、转移代币等等
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, Balance};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde_json::json;
//Native Types u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, Vec<T>, HashMap<K,V> ...
//values larger than 52 bytes (such as u64 and u128), for which string-like alternatives
//overflow-checks=true Cargo.toml
const DEFAULT_MESSAGE: &str = "Hello";

//  Collections contract's attributes (state), Always prefer SDK collections over native 
/*
vector: Vector::new(b"vec-uid-1".to_vec()),
map: LookupMap::new(b"map-uid-1".to_vec()),
set: UnorderedSet::new(b"set-uid-1".to_vec()),
tree: TreeMap::new(b"tree-uid-1".to_vec()),
*/

// Bob 合约
const HELLO_NEAR: &str = "hello-nearverse.testnet";
const NO_DEPOSIT: u128 = 0;
const CALL_GAS: Gas = Gas(5_000_000_000_000);

const MIN_STORAGE: Balance = 100_000_000_000_000_000_000_000; //0.1 N
//const HELLO_CODE: &[u8] = include_bytes!("./hello.wasm");

// Internal Structures 包括非bindings的内部结构和 bindings的合约结构
// NEAR Bindgen bindings装饰器  Define the contract structure 合约的结构，Borsh 序列化状态存储， json 序列化作为方法的输入输出
// NEAR Bindgen decorator/macro 将代码转换为有效的NEAR合约
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    //KEY-VALUE STORAGE 键值存储, SDK通过borsh进行了抽象，业务层面dapps don't care
    greeting: String,
    pub beneficiary: AccountId,
    pub donations: UnorderedMap<AccountId, u128>,
}

impl Default for Contract {
    // The default trait with which to initialize the contract
    // 默认的合约状态属性greeting为 Hello， 直到init或者method方法写入
    fn default() -> Self {
        Self {
            greeting: DEFAULT_MESSAGE.to_string(),
            beneficiary: "v1.faucet.nonofficial.testnet".parse().unwrap(),
            // structures (Vectors, Sets, Maps and Trees) unique prefix进行初始化, 用于在序列化状态中识别结构的键
            //Nesting of Objects RS支持对象嵌套
            donations: UnorderedMap::new(b"d"),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // Public Methods init/view/call 三类， view方法默认有200Tgas
    // 在状态已经初始化过了, 再调用这个init就会报错的， 状态只能搞一次
    //init默认是public的, 必须修饰为 private 或在部署时批量调用初始化
    #[init]
    #[private] // Public - but only callable by env::current_account_id()
    pub fn init(beneficiary: AccountId) -> Self {
        Self {
            greeting: "hala".to_string(),
            beneficiary,
            donations: UnorderedMap::new(b"d"),
        }
    }
    // Public: Returns the stored greeting, defaulting to 'Hello'
    // view方法，任何人都可以自由查看 
    pub fn get_greeting(&self) -> String {
        return self.greeting.clone();
    }

    // Public: Takes a greeting, such as 'howdy', and records it
    // changable 方法， 修改状态必须账户去签名交易才行
    pub fn set_greeting(&mut self, greeting: String) {
        // Record a log permanently to the blockchain!
        log!("Saving greeting {}", greeting);
        self.greeting = greeting;
    }
    // Cross Contract Call a low level way of calling other methods
    // 两个Promises new, cross address, method, encoded args, gas(从attached Gas扣除), near(从contract’s balance扣除)
    //then则是cb结果, 如果你愿意可以发送到任何合约
    //跨合同调用和回调都不会立即执行
    //cross-contract call method finishes correctly will execute 1 or 2 blocks
    //callback will then execute 1 or 2 blocks after the external method finishes (correctly or not)
    pub fn call_method(&self){
        let args = json!({ "message": "howdy".to_string() })
                  .to_string().into_bytes().to_vec();
    
        Promise::new(HELLO_NEAR.parse().unwrap())
        .function_call("set_greeting".to_string(), args, NO_DEPOSIT, CALL_GAS)
        .then(
          Promise::new(env::current_account_id())
          .function_call("callback".to_string(), Vec::new(), NO_DEPOSIT, CALL_GAS)
        );
    }
    
    // Private Methods 方法保持公开，但只能由合约账户调用, 如跨合约回调，设置Owner
    #[private]
      pub fn callback(&self, #[callback_result] result: Result<(), near_sdk::PromiseError>){
        // this method can only be called by the contract's account
        if result.is_err(){
            log!("Something went wrong")
            // <TODO:> 状态不会回滚, 手动回滚调用前进行了任何状态更改（即更改或存储数据）
            // attached NEAR 到call则需要reset
            // 所有调用都是异步的，独立的，确保在调用和回调之间不要让合约处于可利用空子状态
            // 如果外部调用失败，必须在回调中手动回滚状态更改
        }else{
            log!("Message changed")
        }
    }

    // Cross-Contract Calls are Asynchronous, high level way

    #[private]
    pub fn set_owner(&mut self) { 
        /* public, panics when caller is not the contract's account */ 
    }

    // call的时候 附加资金attaches money, 有转账的方法
    #[payable]
    pub fn deposit_and_stake(&mut self ){
        // this method can receive money from the user
    }

    //Input & Return Types 通过接口使用JSON序列化抽象
    // 优先选择输入和返回类型中的 native types， 用 strings 替换 u64 / u128
    // near_sdk::json_types::{U64, I64, U128, I128} 表示, json的大整数最大也是52 bytes


    /*
    Actions 事务原子性保证 Accounts操作相关都语义化为Actions Operation链
    在作用于同一合约时可以被批处理batch在一起。批处理操作作为一个单元：
    它们在同一个收据回执中执行，如果有任何失败，那么它们都会被回滚
     */

    // costs ~0.45 TGas, 在genesis配置
    // 接收者不存在会转账失败,  留下些余额支付未来的存储需求
    pub fn transfer(&self, to: AccountId, amount: Balance){
        Promise::new(to).transfer(amount);
    }

    // 创建直接子账户sub.jong, namespace 设计, 有独立的密钥对, 方便组织账户而已
    pub fn create_sub_account(&self, prefix: String){
        let account_id = prefix + "." + &env::current_account_id().to_string();
        Promise::new(account_id.parse().unwrap())
        .create_account() // 默认是锁定账户, 没有蜜月对
        .transfer(MIN_STORAGE);
    }
    pub fn create_hello(&self, prefix: String, public_key: near_sdk::PublicKey){
        let account_id = prefix + "." + &env::current_account_id().to_string();
        Promise::new(account_id.parse().unwrap())
        .create_account()
        .transfer(MIN_STORAGE)
        //.deploy_contract(HELLO_CODE.to_vec())
        .add_full_access_key(public_key);
    }
    //root contracts Creating Other Accounts
    pub fn create_other_account(&self, account_id: String, public_key: String){
        let args = json!({
                    "new_account_id": account_id,
                    "new_public_key": public_key,
                  }).to_string().into_bytes().to_vec();
    
        // Use "near" to create mainnet accounts
        Promise::new("testnet".parse().unwrap())
        .function_call("create_account".to_string(), args, MIN_STORAGE, CALL_GAS);
    }

    //Delete Account, Token loss
    // beneficiary account受益人账户不存在, dispersed among validators
    // delete 来尝试fund新账户，导致账户不存在 tokens will be lost
    pub fn create_delete(&self, prefix: String, beneficiary: AccountId){
        let account_id = prefix + "." + &env::current_account_id().to_string();
        Promise::new(account_id.parse().unwrap())
        .create_account()
        .transfer(MIN_STORAGE)
        .delete_account(beneficiary);
    }
    
    pub fn self_delete(beneficiary: AccountId){
        Promise::new(env::current_account_id())
        .delete_account(beneficiary);
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_greeting(), "Hello".to_string());
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }
}
