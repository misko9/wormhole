
package ictest

import (
	"encoding/json"
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
func TestTokenfactory(t *testing.T) {
	t.Parallel()

	// Base setup
	guardians := guardians.CreateValSet(t, numVals)
	chains := CreateChains2(t, *guardians)
	ctx, _, _ := BuildInterchain2(t, chains)

	// Chains
	wormchain := chains[0].(*cosmos.CosmosChain)
	osmosis := chains[1].(*cosmos.CosmosChain)
	t.Log("simd.GetHostRPCAddress()", wormchain.GetHostRPCAddress())

	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000_000), wormchain, osmosis, osmosis)
	_ = users[0] // Wormchain user
	osmoUser1 := users[1]
	osmoUser2 := users[2]

	ibcHooksCodeId, err := osmosis.StoreContract(ctx, osmoUser1.KeyName, "./contracts/ibc_hooks.wasm")
	require.NoError(t, err)
	fmt.Println("IBC hooks code id: ", ibcHooksCodeId)

	ibcHooksContractAddr, err := osmosis.InstantiateContract(ctx, osmoUser1.KeyName, ibcHooksCodeId, "{}", true)
	require.NoError(t, err)
	fmt.Println("IBC hooks contract addr: ", ibcHooksContractAddr)

	tokenfactoryIbcCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/tokenfactory_ibc.wasm", guardians)
	fmt.Println("Tokenfactory ibc contract code id: ", tokenfactoryIbcCodeId)

	tokenfactoryIbcContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", tokenfactoryIbcCodeId, "tokenfactory_ibc", "{}", guardians)
	fmt.Println("Tokenfactory ibc contract address: ", tokenfactoryIbcContractAddr)

	ibcHooksMsg := helpers.CreateIbcHooksMsg(t, ibcHooksContractAddr, osmoUser2.Bech32Address(osmosis.Config().Bech32Prefix))
	tfMsg := CreateTfPayload(t, ibcHooksContractAddr, "3456789", string(ibcHooksMsg))
	_, err = wormchain.ExecuteContract(ctx, "faucet", tokenfactoryIbcContractAddr, tfMsg)

	// wait for transfer
	err = testutil.WaitForBlocks(ctx, 4, wormchain)
	require.NoError(t, err)
	
	coins, err := wormchain.AllBalances(ctx, tokenfactoryIbcContractAddr)
	require.NoError(t, err)
	fmt.Println("Ibc Gateway contract coins: ", coins)
	
	coins, err = osmosis.AllBalances(ctx, osmoUser1.Bech32Address(osmosis.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Osmo user1 coins: ", coins)

	coins, err = osmosis.AllBalances(ctx, osmoUser2.Bech32Address(osmosis.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Osmo user2 coins: ", coins)
}

type TfExecute struct {
	CreateTokenAndSend CreateTokenAndSend `json:"create_token_and_send"`
}

type CreateTokenAndSend struct {
	Contract string `json:"contract"`
	Amount string `json:"amount"`
	Payload string `json:"payload"`
}

func CreateTfPayload(t *testing.T, contract string, amount string, payload string) string {
	msg := TfExecute {
		CreateTokenAndSend: CreateTokenAndSend{
			Contract: contract,
			Amount: amount,
			Payload: payload,
		},
	}

	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return string(msgBz)
}