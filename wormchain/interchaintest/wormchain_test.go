package ictest

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v4/testutil"
	"github.com/stretchr/testify/require"

	"github.com/wormhole-foundation/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormchain/interchaintest/helpers"
)

// TestWormchain runs through a simple test case for each deliverable
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

	coreContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/wormhole_core.wasm", guardians)
	fmt.Println("Core contract code id: ", coreContractCodeId)
	
	coreInstantiateMsg := helpers.CoreContractInstantiateMsg(t, wormchainConfig, guardians)
	coreContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", coreContractCodeId, "wormhole_core", coreInstantiateMsg, guardians)
	fmt.Println("Core contract address: ", coreContractAddr)

	wrappedAssetCodeId := helpers.StoreContract(t, ctx, wormchain,"faucet", "./contracts/cw20_wrapped_2.wasm", guardians)
	fmt.Println("CW20 wrapped_2 code id: ", wrappedAssetCodeId)

	tbContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/token_bridge.wasm", guardians)
	fmt.Println("Token bridge contract code id: ", tbContractCodeId)

	tbInstantiateMsg:= helpers.TbContractInstantiateMsg(t, wormchainConfig, coreContractAddr, wrappedAssetCodeId)
	tbContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", tbContractCodeId, "token_bridge", tbInstantiateMsg, guardians)
	fmt.Println("Token bridge contract address: ", tbContractAddr)

	// Add a bridged token
	// Send a bridged token to wormchain using token bridge and deposited to an address
	// Send a bridged token out of wormchain

	// IBC shim required:
	// Send a bridged token to wormchain, over ibc to foreign cosmos chain (deposited to addr)
	// Send a bridged token to wormchain, over ibc to a foreign cosmos chain's contract

	// IBC hooks required:
	// Send a bridged token from foreign cosmos chain, over ibc to wormchain and out of wormchain (stretch, nice-to-have)

	// PFM required:
	// Send a bridged token from a foreign cosmos chain, through wormchain, to a second foreign cosmos chain (deposited to addr)
	// Send a bridged token from a foreign cosmos chain, through wormchain, to a second foreign cosmos chain's contract

	// Out of scope:
	// Send a cosmos chain native asset to wormchain for external chain consumption

	err := testutil.WaitForBlocks(ctx, 2, wormchain)
	require.NoError(t, err)
}

