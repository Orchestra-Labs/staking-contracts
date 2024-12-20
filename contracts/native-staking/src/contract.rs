use cw_storage_plus::Item;
use sylvia::contract;
use sylvia::ctx::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::cw_schema::cw_serde;
#[cfg(not(feature = "library"))]
use sylvia::cw_std::Empty;
use sylvia::cw_std::{Addr, Response, StdResult};
use sylvia::types::{CustomMsg, CustomQuery};
use cw_utils::Duration;

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct NativeToken {
    pub denom: Option<String>,
    pub decimals: u8,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct InstantiateMsgData {
    pub owner: Option<Addr>,
    pub native_token: NativeToken,
    pub unstaking_duration: Option<Duration>,
}

pub struct CounterContract<E, Q> {
    pub count: Item<u64>,
    pub owner: Item<Addr>,
    pub native_token: Item<NativeToken>,
    pub unstaking_duration: Item<Duration>,
    _phantom: std::marker::PhantomData<(E, Q)>,
}

#[cfg_attr(not(feature = "library"), sylvia::entry_points(generics<Empty, Empty>))]
#[contract]
#[sv::custom(msg = E, query = Q)]
impl<E, Q> CounterContract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    pub const fn new() -> Self {
        Self {
            count: Item::new("count"),
            owner: Item::new("owner"),
            native_token: Item::new("native_token"),
            unstaking_duration: Item::new("unstaking_duration"),
            _phantom: std::marker::PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx<Q>, data: InstantiateMsgData) -> StdResult<Response<E>> {
        let owner = data.owner.unwrap_or(ctx.info.sender);
        ctx.deps.api.addr_validate(owner.as_str())?;
        self.owner.save(ctx.deps.storage, &owner)?;

        self.native_token.save(ctx.deps.storage, &data.native_token)?;

        let unbounding_duration = data.unstaking_duration.unwrap_or(Duration::Time(0));
        self.unstaking_duration.save(ctx.deps.storage, &unbounding_duration)?;

        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn increment(&self, ctx: ExecCtx<Q>) -> StdResult<Response<E>> {
        self.count
            .update(ctx.deps.storage, |count| -> StdResult<u64> {
                Ok(count + 1)
            })?;
        Ok(Response::new())
    }

    #[sv::msg(query)]
    fn count(&self, ctx: QueryCtx<Q>) -> StdResult<CountResponse> {
        let count = self.count.load(ctx.deps.storage)?;
        Ok(CountResponse { count })
    }
}

#[cw_serde(crate = "sylvia")]
pub struct CountResponse {
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use sylvia::cw_multi_test::IntoAddr;
    use sylvia::cw_std::testing::{message_info, mock_dependencies, mock_env};
    use sylvia::cw_std::Empty;

    // Unit tests don't have to use a testing framework for simple things.
    //
    // For more complex tests (particularly involving cross-contract calls), you
    // may want to check out `cw-multi-test`:
    // https://github.com/CosmWasm/cw-multi-test
    #[test]
    fn init() {
        let sender = "alice".into_addr();
        let contract = CounterContract::<Empty, Empty>::new();
        let mut deps = mock_dependencies();
        let ctx = InstantiateCtx::from((deps.as_mut(), mock_env(), message_info(&sender, &[])));
        contract.instantiate(ctx).unwrap();

        // We're inspecting the raw storage here, which is fine in unit tests. In
        // integration tests, you should not inspect the internal state like this,
        // but observe the external results.
        assert_eq!(0, contract.count.load(deps.as_ref().storage).unwrap());
    }

    #[test]
    fn query() {
        let sender = "alice".into_addr();
        let contract = CounterContract::<Empty, Empty>::new();
        let mut deps = mock_dependencies();
        let ctx = InstantiateCtx::from((deps.as_mut(), mock_env(), message_info(&sender, &[])));
        contract.instantiate(ctx).unwrap();

        let ctx = QueryCtx::from((deps.as_ref(), mock_env()));
        let res = contract.count(ctx).unwrap();
        assert_eq!(0, res.count);
    }

    #[test]
    fn inc() {
        let sender = "alice".into_addr();
        let contract = CounterContract::<Empty, Empty>::new();
        let mut deps = mock_dependencies();
        let ctx = InstantiateCtx::from((deps.as_mut(), mock_env(), message_info(&sender, &[])));
        contract.instantiate(ctx).unwrap();

        let ctx = ExecCtx::from((deps.as_mut(), mock_env(), message_info(&sender, &[])));
        contract.increment(ctx).unwrap();

        let ctx = QueryCtx::from((deps.as_ref(), mock_env()));
        let res = contract.count(ctx).unwrap();
        assert_eq!(1, res.count);
    }
}
