package helpers

import (
	"encoding/json"
	"strconv"
	"testing"

	"github.com/wormhole-foundation/wormhole/sdk/vaa"
	"github.com/stretchr/testify/require"
	"github.com/strangelove-ventures/interchaintest/v4/ibc"
)

type TbInstantiateMsg struct {
	GovChain uint16 `json:"gov_chain"`
	GovAddress []byte `json:"gov_address"`

	WormholeContract string `json:"wormhole_contract"`
	WrappedAssetCodeId uint64 `json:"wrapped_asset_code_id"`

	ChainId  uint16 `json:"chain_id"`
	NativeDenom string `json:"native_denom"`
	NativeSymbol string `json:"native_symbol"`
	NativeDecimals uint8 `json:"native_decimals"`
}

func TbContractInstantiateMsg(t *testing.T, cfg ibc.ChainConfig, whContract string, wrappedAssetCodeId string) string {
	codeId, err := strconv.ParseUint(wrappedAssetCodeId, 10, 64)
	require.NoError(t, err)

	msg := TbInstantiateMsg{
		GovChain: uint16(vaa.GovernanceChain),
		GovAddress: vaa.GovernanceEmitter[:],
		WormholeContract: whContract,
		WrappedAssetCodeId: codeId,
		ChainId: uint16(vaa.ChainIDWormchain),
		NativeDenom: cfg.Denom,
		NativeSymbol: "WORM",
		NativeDecimals: 6,
	}
	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return string(msgBz)
}