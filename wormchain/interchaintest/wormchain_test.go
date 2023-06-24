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
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
)

var gaiaChainID = 11

// TestWormchain runs through a simple test case for each deliverable
func TestWormchain(t *testing.T) {
	t.Parallel()

	// Base setup
	guardians := guardians.CreateValSet(t, numVals)
	chains := CreateChains(t, *guardians)
	ctx := BuildInterchain(t, chains)

	// Chains
	wormchain := chains[0].(*cosmos.CosmosChain)
	gaia := chains[1].(*cosmos.CosmosChain)
	t.Log("simd.GetHostRPCAddress()", wormchain.GetHostRPCAddress())

	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000_000), wormchain, gaia)
	_ = users[0] // Wormchain user
	gaiaUser := users[1]

	coreContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/wormhole_core.wasm", guardians)
	fmt.Println("Core contract code id: ", coreContractCodeId)
	
	coreInstantiateMsg := helpers.CoreContractInstantiateMsg(t, wormchainConfig, guardians)
	coreContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", coreContractCodeId, "wormhole_core", coreInstantiateMsg, guardians)
	fmt.Println("Core contract address: ", coreContractAddr)

	/*queryMsg := QueryMsg{GuardianSetInfo: &struct{}{}}
	var queryRsp QueryRsp
	err := wormchain.QueryContract(ctx, coreContractAddr, queryMsg, &queryRsp)
	require.NoError(t, err)*/

	wrappedAssetCodeId := helpers.StoreContract(t, ctx, wormchain,"faucet", "./contracts/cw20_wrapped_2.wasm", guardians)
	fmt.Println("CW20 wrapped_2 code id: ", wrappedAssetCodeId)

	tbContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/token_bridge.wasm", guardians)
	fmt.Println("Token bridge contract code id: ", tbContractCodeId)

	tbInstantiateMsg:= helpers.TbContractInstantiateMsg(t, wormchainConfig, coreContractAddr, wrappedAssetCodeId)
	tbContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", tbContractCodeId, "token_bridge", tbInstantiateMsg, guardians)
	fmt.Println("Token bridge contract address: ", tbContractAddr)

	// Add a bridged token
	tbRegisterChainMsg := helpers.TbRegisterChainMsg(t, 123, "123TokenBridge", guardians)
	_, err := wormchain.ExecuteContract(ctx, "faucet", tbContractAddr, string(tbRegisterChainMsg))
	require.NoError(t, err)

	name := "Wrapped BTC"
	symbol := "XBTC"
	assetAddr := "0xXBTC"
	assetChainID := uint16(123)
	
	tbRegisterForeignAssetMsg := helpers.TbRegisterForeignAsset(t, assetAddr, assetChainID, "123TokenBridge", 6, symbol, name, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", tbContractAddr, string(tbRegisterForeignAssetMsg))
	require.NoError(t, err)
	
	//cw20InstantiateMsg := helpers.Cw20ContractInstantiateMsg(t, name, symbol, assetChainID, assetAddr, 6, tbContractAddr)
	//_ = helpers.InstantiateContract(t, ctx, wormchain, "faucet", wrappedAssetCodeId, symbol, cw20InstantiateMsg, guardians)

	ibcGatewayCodeId := helpers.StoreContract(t, ctx, wormchain,"faucet", "./contracts/ibc_gateway.wasm", guardians)
	fmt.Println("ibc_gateway code id: ", ibcGatewayCodeId)

	ibcGwInstantiateMsg := helpers.IbcGwContractInstantiateMsg(t, tbContractAddr)
	ibcGwContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", ibcGatewayCodeId, "ibc_gateway", ibcGwInstantiateMsg, guardians)
	fmt.Println("Ibc gateway contract address: ", ibcGwContractAddr)

	simplePayload := helpers.CreateGatewayIbcTokenBridgePayloadSimple(t, uint16(gaiaChainID), gaiaUser.Bech32Address(gaia.Config().Bech32Prefix), 0, 1)
	externalSender := []byte{1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8 ,1, 2, 3, 4, 5, 6, 7, 8}
	payload3 := helpers.CreatePayload3(wormchain.Config(), 1231245, assetAddr, assetChainID, ibcGwContractAddr, uint16(vaa.ChainIDWormchain), externalSender, simplePayload)
	completeTransferAndConvertMsg := helpers.IbcGwCompleteTransferAndConvertMsg(t, 123, "123TokenBridge", payload3, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", ibcGwContractAddr, completeTransferAndConvertMsg)

	// wait for transfer
	err = testutil.WaitForBlocks(ctx, 5, wormchain)
	require.NoError(t, err)
	
	coins, err := wormchain.AllBalances(ctx, ibcGwContractAddr)
	require.NoError(t, err)
	fmt.Println("Ibc Gateway contract coins: ", coins)

	coins, err = gaia.AllBalances(ctx, gaiaUser.Bech32Address(gaia.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Gaia user coins: ", coins)
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

	err = testutil.WaitForBlocks(ctx, 2, wormchain)
	require.NoError(t, err)
}

type QueryMsg struct {
	GuardianSetInfo *struct{} `json:"guardian_set_info,omitempty"`
}

type QueryRsp struct {
	Data *QueryRspObj `json:"data,omitempty"`
}

type QueryRspObj struct {
	GuardianSetIndex uint32 `json:"guardian_set_index"`
	Addresses []helpers.GuardianAddress `json:"addresses"`
}