use frame_support::{ensure, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Order {
    pub id: u64,
    pub user: T::AccountId,
    pub token: T::Hash,
    pub amount: u64,
    pub price: u64,
    pub is_buy: bool,
    pub is_filled: bool,
}

decl_storage! {
    trait Store for Module<T: Trait> as DEX {
        pub Orders get(fn orders): map hasher(blake2_128_concat) u64 => Option<Order>;
        pub OrderCount get(fn order_count): u64;
        pub Tokens get(fn tokens): map hasher(twox_64_concat) T::Hash => Option<T>;
        pub TokenBalances get(fn token_balances): map hasher(blake2_128_concat) (T::AccountId, T::Hash) => u64;
    }
}

impl<T: Trait> Module<T> {
    #[weight = 10_000]
    pub fn create_order(
        origin,
        token: T::Hash,
        amount: u64,
        price: u64,
        is_buy: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let id = Self::order_count();
        let order = Order {
            id,
            user: who.clone(),
            token,
            amount,
            price,
            is_buy,
            is_filled: false,
        };
        Orders::insert(id, order.clone());
        OrderCount::put(id + 1);

        Self::deposit_event(RawEvent::NewOrder(order));
        Ok(())
    }

    pub fn execute_order(order_id: u64, amount: u64) -> DispatchResult {
        let order = Self::orders(order_id).ok_or_else(|| "Order does not exist")?;
        ensure!(!order.is_filled, "Order is already filled");

        if order.is_buy {
            let total_cost = amount * order.price;
            let token_balance = Self::token_balances(&(order.user.clone(), order.token.clone()));
            ensure!(
                token_balance >= total_cost,
                "Order user does not have enough balance to execute buy order"
            );

            // exchange tokens
            let buyer = ensure_signed(order.user.clone())?;
            let seller =
                ensure_signed(Self::orders(orders_id).unwrap().user.clone().into_account()?);

            let buyer_token_balance = Self::token_balances(&(buyer.clone(), order.token.clone()));
            let seller_token_balance =
                Self::token_balances(&(seller.clone(), order.token.clone()));

            TokenBalances::insert(
                &(buyer.clone(), order.token.clone()),
                buyer_token_balance + amount,
            );
            TokenBalances::insert(
                &(seller.clone(), order.token.clone()),
                seller_token_balance - amount,
            );
            Self::deposit_event(RawEvent::OrderMatched(order_id, amount, order.price));
        } else {
            let token_balance = Self::token_balances(&(order.user.clone(), order.token.clone()));
            ensure!(
                token_balance >= amount,
                "Order user does not have enough balance to execute sell order"
            );

            // exchange tokens
            let seller = ensure_signed(order.user.clone())?;
            let buyer =
                ensure_signed(Self::orders(orders_id).unwrap().user.clone().into_account()?);

            let buyer_token_balance = Self::token_balances(&(buyer.clone(), order.token.clone()));
            let seller_token_balance =
                Self::token_balances(&(seller.clone(), order.token.clone()));

            TokenBalances::insert(
                &(buyer.clone(), order.token.clone()),
                buyer_token_balance + amount,
            );
            TokenBalances::insert(
                &(seller.clone(), order.token.clone()),
                seller_token_balance - amount,
            );
            Self::deposit_event(RawEvent::OrderMatched(order_id, amount, order.price));
        }

        Ok(())
    }
}
