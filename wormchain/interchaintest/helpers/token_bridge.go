package helpers

import (
	"encoding/json"
	"strconv"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/stretchr/testify/require"
	"github.com/wormhole-foundation/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
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

type TbExecuteMsg struct {
	SubmitVaa TbSubmitVaaMsg `json:"submit_vaa,omitempty"`
}

type TbSubmitVaaMsg struct {
	Data []byte `json:"data"`
}

func TbRegisterChainMsg(t *testing.T, chainID uint16, emitterAddr string, guardians *guardians.ValSet) []byte {
	emitterBz := [32]byte{}
	eIndex := 32
	for i := len(emitterAddr); i > 0; i-- {
		emitterBz[eIndex-1] = emitterAddr[i-1]
		eIndex--
	}
	bodyTbRegisterChain := vaa.BodyTokenBridgeRegisterChain{
		Module: "TokenBridge",
		ChainID: vaa.ChainID(chainID),
		EmitterAddress: vaa.Address(emitterBz),
	}

	payload := bodyTbRegisterChain.Serialize()
	v := generateVaa(0, guardians, vaa.ChainID(vaa.GovernanceChain), payload)
	vBz, err := v.Marshal()
	require.NoError(t, err)
		
	msg := TbExecuteMsg{
		SubmitVaa: TbSubmitVaaMsg{
			Data: vBz,
		},
	}

	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return msgBz
}