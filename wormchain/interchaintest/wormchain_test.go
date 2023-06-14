package ictest

import (
	"encoding/base64"
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v7"
	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v7/testutil"
	"github.com/stretchr/testify/require"

	"github.com/wormhole-foundation/wormhole/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormhole/wormchain/interchaintest/helpers"
)

func TestValAddr(t *testing.T) {
	valAddr := MustAccAddressFromBech32("wormhole1wqwywkce50mg6077huy4j9y8lt80943ks5udzr", "wormhole").Bytes()
	valAddrB64 := base64.RawStdEncoding.EncodeToString(valAddr)
	fmt.Println("Val: ", valAddrB64)
}

// TestWormchain runs through a simple set of tests to ensure things are working
func TestWormchain(t *testing.T) {
	t.Parallel()

	// Base setup
	guardians := guardians.CreateValSet(t, numVals)
	chains := CreateSingleWormchain(t, *guardians)
	ctx := BuildInitialChain(t, chains)

	// Chains
	wormchain := chains[0].(*cosmos.CosmosChain)
	t.Log("simd.GetHostRPCAddress()", wormchain.GetHostRPCAddress())

	_ = interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000_000), wormchain)
	//user := users[0]

	coreContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/core.wasm", guardians)
	fmt.Println("Core contract code id: ", coreContractCodeId)

//	msg := fmt.Sprintf(`{}`)
//	_, contract := helpers.SetupContract(t, ctx, simd, user.KeyName(), "contracts/wormchain.wasm", msg)
//	t.Log("coreContract", contract)

//	verifyContractEntryPoints(t, ctx, simd, user, contract)

	// Either add fake guardian/validator or figure out how to submit a tx from a val
	// Add a new allowed account (this will be needed for the relayer) or is this done in genesis??
	// Store and initialize the core bridge contract (how to initialize?)
	// Store and initialize the token bridge contract (how to initialize?)
	// Add a new token



	err := testutil.WaitForBlocks(ctx, 200, wormchain)
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
