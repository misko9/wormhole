package ictest

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/strangelove-ventures/interchaintest/v4/testutil"
	"github.com/stretchr/testify/require"

	"github.com/wormhole-foundation/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormchain/interchaintest/helpers"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
)

var (
	gaiaChainID = uint16(11)
	osmoChainID = uint16(12)

	externalChainId = uint16(123)
	externalChainEmitterAddr = "0x123EmitterAddress"

	asset1Name = "Wrapped BTC"
	asset1Symbol = "XBTC"
	asset1ContractAddr = "0xXBTC"
	asset1ChainID = externalChainId
	asset1Decimals = uint8(6)
)

// TestWormchain runs through a simple test case for each deliverable
func TestWormchain(t *testing.T) {
	t.Parallel()

	// Base setup
	guardians := guardians.CreateValSet(t, numVals)
	chains := CreateChains(t, *guardians)
	ctx, r, eRep := BuildInterchain(t, chains)


	// Chains
	wormchain := chains[0].(*cosmos.CosmosChain)
	gaia := chains[1].(*cosmos.CosmosChain)
	osmosis := chains[2].(*cosmos.CosmosChain)
	t.Log("simd.GetHostRPCAddress()", wormchain.GetHostRPCAddress())

	osmoToWormChannel, err := ibc.GetTransferChannel(ctx, r, eRep, osmosis.Config().ChainID, wormchain.Config().ChainID)
	wormToOsmoChannel := osmoToWormChannel.Counterparty 
	gaiaToWormChannel, err := ibc.GetTransferChannel(ctx, r, eRep, gaia.Config().ChainID, wormchain.Config().ChainID)
	wormToGaiaChannel := gaiaToWormChannel.Counterparty 
	fmt.Println("Worm to Osmo channel: ", wormToOsmoChannel.ChannelID)
	fmt.Println("Worm to Gaia channel: ", wormToGaiaChannel.ChannelID)
	fmt.Println("Osmo to Worm channel: ", osmoToWormChannel.ChannelID)
	fmt.Println("Gaia to Worm channel: ", gaiaToWormChannel.ChannelID)

	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000_000), wormchain, gaia, osmosis, osmosis)
	_ = users[0] // Wormchain user
	gaiaUser := users[1]
	osmoUser1 := users[2]
	osmoUser2 := users[3]

	codeId, err := osmosis.StoreContract(ctx, osmoUser1.KeyName, "./contracts/cw20_wrapped_2.wasm")
	require.NoError(t, err)
	fmt.Println("Code id: ", codeId)

	_, err = wormchain.SendIBCTransfer(ctx, wormToOsmoChannel.ChannelID, "faucet", ibc.WalletAmount{
		Address: osmoUser2.Bech32Address(osmosis.Config().Bech32Prefix),
		Amount: 11122233,
		Denom: wormchain.Config().Denom,
	}, ibc.TransferOptions{})
	require.NoError(t, err)

	// Store wormhole core contract
	coreContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/wormhole_core.wasm", guardians)
	fmt.Println("Core contract code id: ", coreContractCodeId)
	
	// Instantiate wormhole core contract
	coreInstantiateMsg := helpers.CoreContractInstantiateMsg(t, wormchainConfig, guardians)
	coreContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", coreContractCodeId, "wormhole_core", coreInstantiateMsg, guardians)
	fmt.Println("Core contract address: ", coreContractAddr)

	// Store cw20_wrapped_2 contract
	wrappedAssetCodeId := helpers.StoreContract(t, ctx, wormchain,"faucet", "./contracts/cw20_wrapped_2.wasm", guardians)
	fmt.Println("CW20 wrapped_2 code id: ", wrappedAssetCodeId)

	// Store token bridge contract
	tbContractCodeId := helpers.StoreContract(t, ctx, wormchain, "faucet", "./contracts/token_bridge.wasm", guardians)
	fmt.Println("Token bridge contract code id: ", tbContractCodeId)

	// Instantiate token bridge contract
	tbInstantiateMsg:= helpers.TbContractInstantiateMsg(t, wormchainConfig, coreContractAddr, wrappedAssetCodeId)
	tbContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", tbContractCodeId, "token_bridge", tbInstantiateMsg, guardians)
	fmt.Println("Token bridge contract address: ", tbContractAddr)

	// Register a new external chain
	tbRegisterChainMsg := helpers.TbRegisterChainMsg(t, externalChainId, externalChainEmitterAddr, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", tbContractAddr, string(tbRegisterChainMsg))
	require.NoError(t, err)

	// Register a new foreign asset (Asset1) originating on externalChain
	tbRegisterForeignAssetMsg := helpers.TbRegisterForeignAsset(t, asset1ContractAddr, asset1ChainID, externalChainEmitterAddr, asset1Decimals, asset1Symbol, asset1Name, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", tbContractAddr, string(tbRegisterForeignAssetMsg))
	require.NoError(t, err)
	
	// Store ibc gateway contract
	ibcGatewayCodeId := helpers.StoreContract(t, ctx, wormchain,"faucet", "./contracts/ibc_gateway.wasm", guardians)
	fmt.Println("ibc_gateway code id: ", ibcGatewayCodeId)

	// Instantiate ibc gateway contract
	ibcGwInstantiateMsg := helpers.IbcGwContractInstantiateMsg(t, tbContractAddr, coreContractAddr)
	ibcGwContractAddr := helpers.InstantiateContract(t, ctx, wormchain, "faucet", ibcGatewayCodeId, "ibc_gateway", ibcGwInstantiateMsg, guardians)
	fmt.Println("Ibc gateway contract address: ", ibcGwContractAddr)

	// Allowlist worm/osmo chain id / channel
	wormOsmoAllowlistMsg := helpers.SubmitUpdateChainToChannelMapMsg(t, osmoChainID, wormToOsmoChannel.ChannelID, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", ibcGwContractAddr, wormOsmoAllowlistMsg)

	// Allowlist worm/gaia chain id / channel
	wormGaiaAllowlistMsg := helpers.SubmitUpdateChainToChannelMapMsg(t, gaiaChainID, wormToGaiaChannel.ChannelID, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", ibcGwContractAddr, wormGaiaAllowlistMsg)

	// Create and process a simple ibc payload3: Transfers 1.231245 of asset1 from external chain through wormchain to gaia user
	simplePayload := helpers.CreateGatewayIbcTokenBridgePayloadSimple(t, gaiaChainID, gaiaUser.Bech32Address(gaia.Config().Bech32Prefix), 0, 1)
	externalSender := []byte{1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8 ,1, 2, 3, 4, 5, 6, 7, 8}
	payload3 := helpers.CreatePayload3(wormchain.Config(), 1231245, asset1ContractAddr, asset1ChainID, ibcGwContractAddr, uint16(vaa.ChainIDWormchain), externalSender, simplePayload)
	completeTransferAndConvertMsg := helpers.IbcGwCompleteTransferAndConvertMsg(t, externalChainId, externalChainEmitterAddr, payload3, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", ibcGwContractAddr, completeTransferAndConvertMsg)

	// Create and process a contract controlled ibc payload3: Transfers 1.456789 of asset1 from external chain through wormchain to gaia user (eventually to ibcHooksSimd contract)
	contractControlledPayload := helpers.CreateGatewayIbcTokenBridgePayloadContract(t, gaiaChainID, gaiaUser.Bech32Address(gaia.Config().Bech32Prefix), []byte("{ContractPayload}"), 1)
	payload3 = helpers.CreatePayload3(wormchain.Config(), 1456789, asset1ContractAddr, asset1ChainID, ibcGwContractAddr, uint16(vaa.ChainIDWormchain), externalSender, contractControlledPayload)
	completeTransferAndConvertMsg = helpers.IbcGwCompleteTransferAndConvertMsg(t, externalChainId, externalChainEmitterAddr, payload3, guardians)
	_, err = wormchain.ExecuteContract(ctx, "faucet", ibcGwContractAddr, completeTransferAndConvertMsg)

	// wait for transfer
	err = testutil.WaitForBlocks(ctx, 3, wormchain)
	require.NoError(t, err)
	
	coins, err := wormchain.AllBalances(ctx, ibcGwContractAddr)
	require.NoError(t, err)
	fmt.Println("Ibc Gateway contract coins: ", coins)

	coins, err = gaia.AllBalances(ctx, gaiaUser.Bech32Address(gaia.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Gaia user coins: ", coins)
	
	coins, err = osmosis.AllBalances(ctx, osmoUser1.Bech32Address(osmosis.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Osmo user1 coins: ", coins)

	coins, err = osmosis.AllBalances(ctx, osmoUser2.Bech32Address(osmosis.Config().Bech32Prefix))
	require.NoError(t, err)
	fmt.Println("Osmo user2 coins: ", coins)

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