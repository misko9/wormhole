package ictest

import (
	"testing"

	"github.com/strangelove-ventures/interchaintest/v7"
	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v7/testutil"
	"github.com/stretchr/testify/require"
)

// TestWormchain runs through a simple set of tests to ensure things are working
func TestWormchain(t *testing.T) {
	t.Parallel()

	// Base setup
	chains := CreateSingleWormchain(t)
	ctx := BuildInitialChain(t, chains)

	// Chains
	wormchain := chains[0].(*cosmos.CosmosChain)
	t.Log("simd.GetHostRPCAddress()", wormchain.GetHostRPCAddress())

	_ = interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000_000), wormchain)
	//user := users[0]

//	msg := fmt.Sprintf(`{}`)
//	_, contract := helpers.SetupContract(t, ctx, simd, user.KeyName(), "contracts/wormchain.wasm", msg)
//	t.Log("coreContract", contract)

//	verifyContractEntryPoints(t, ctx, simd, user, contract)


	err := testutil.WaitForBlocks(ctx, 2, wormchain)
	require.NoError(t, err)
}

/*func verifyContractEntryPoints(t *testing.T, ctx context.Context, simd *cosmos.CosmosChain, user ibc.Wallet, contract string) {
	queryMsg := helpers.QueryMsg{Owner: &struct{}{}}
	var queryRsp helpers.QueryRsp
	err := simd.QueryContract(ctx, contract, queryMsg, &queryRsp)
	require.NoError(t, err)
	require.Equal(t, user.FormattedAddress(), queryRsp.Data.Address)

	randomAddr := "cosmos10qa7yajp3fp869mdegtpap5zg056exja3chkw5"
	newContractOwnerStruct := helpers.ExecuteMsg{
		ChangeContractOwner: &helpers.ChangeContractOwner{
			NewOwner: randomAddr,
		},
	}
	newContractOwner, err := json.Marshal(newContractOwnerStruct)
	require.NoError(t, err)
	simd.ExecuteContract(ctx, user.KeyName(), contract, string(newContractOwner))

	err = simd.QueryContract(ctx, contract, queryMsg, &queryRsp)
	require.NoError(t, err)
	require.Equal(t, randomAddr, queryRsp.Data.Address)
}*/
