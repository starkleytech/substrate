mod runtime;

use runtime::NodeTemplateChainInfo;
use substrate_test_runner::Node;
use pallet_sudo::Call as SudoCall;
use pallet_balances::Call as BalancesCall;
use sp_keyring::sr25519::Keyring::{Alice, Bob};
use sp_runtime::{traits::IdentifyAccount, MultiSigner};
use node_runtime::Event;
use pallet_balances::RawEvent;

#[test]
fn test_force_transfer() {
    type Balances = pallet_balances::Module<node_runtime::Runtime>;
    let node = Node::<NodeTemplateChainInfo>::new().unwrap();
    let (alice, bob) = (
        MultiSigner::from(Alice.public()).into_account(),
        MultiSigner::from(Bob.public()).into_account(),
    );
    let (alice_balance, bob_balance) = node.with_state(|| (
        Balances::free_balance(alice.clone()),
        Balances::free_balance(bob.clone()),
    ));
    let balance = alice_balance / 2;

    let balances_call = BalancesCall::force_transfer(alice.clone().into(), bob.clone().into(), balance);
    node.submit_extrinsic(
        SudoCall::sudo(Box::new(balances_call.into())),
        alice
    );
    node.seal_blocks(1);

    let events = node.events()
        .into_iter()
        .filter(|event| {
            match &event.event {
                Event::pallet_balances(RawEvent::Transfer(_, _, _)) => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);

    let new_bob_balance = node.with_state(|| Balances::free_balance(bob.clone()));

    assert_eq!(new_bob_balance, bob_balance + (balance))
}

#[test]
fn test_set_balance() {
    type Balances = pallet_balances::Module<node_runtime::Runtime>;
    let node = Node::<NodeTemplateChainInfo>::new().unwrap();
    let (alice, bob) = (
        MultiSigner::from(Alice.public()).into_account(),
        MultiSigner::from(Bob.public()).into_account(),
    );
    let bob_balance = node.with_state(|| Balances::free_balance(bob.clone()));
    let new_bob_balance = bob_balance * 20;

    let call = BalancesCall::set_balance(bob.clone().into(), new_bob_balance, 0);
    node.submit_extrinsic(
        SudoCall::sudo(Box::new(call.into())),
        alice
    );
    node.seal_blocks(1);

    let events = node.events()
        .into_iter()
        .filter(|event| {
            match &event.event {
                Event::pallet_balances(RawEvent::BalanceSet(_, _, _)) => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);

    let updated_bob_balance = node.with_state(|| Balances::free_balance(bob.clone()));

    assert_eq!(updated_bob_balance, new_bob_balance)
}

#[test]
fn test_transfer_keep_alive() {
    type Balances = pallet_balances::Module<node_runtime::Runtime>;
    let node = Node::<NodeTemplateChainInfo>::new().unwrap();
    let (alice, bob) = (
        MultiSigner::from(Alice.public()).into_account(),
        MultiSigner::from(Bob.public()).into_account(),
    );
    let alice_balance = node.with_state(|| Balances::free_balance(alice.clone()));
    // attempt to send more than the existential deposit
    let balance = alice_balance - (node_runtime::ExistentialDeposit::get() -1 );

    let call = BalancesCall::transfer(bob.clone().into(), balance);
    node.submit_extrinsic(call, alice);
    node.seal_blocks(1);

    // assert that the transaction failed to dispatch
    node.assert_log_line("LiquidityRestrictions");

    let new_balance = node.with_state(|| Balances::free_balance(bob.clone()));

    assert_eq!(alice_balance, new_balance)
}
