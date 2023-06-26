package helpers

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"
)

type Cw20InstantiateMsg struct {
	Name string `json:"name"`
	Symbol string `json:"symbol"`
	AssetChain uint16 `json:"asset_chain"`
	AssetAddr []byte `json:"asset_address"`
	Decimals uint8 `json:"decimals"`
	InitHook InitHook `json:"init_hook"`
}

type InitHook struct {
	Msg []byte `json:"msg"`
	ContractAddr string `json:"contract_addr"`
}

type TbRegisterAssetHookMsg struct {
	RegisterAssetHook RegisterAssetHook `json:"register_asset_hook,omitempty"`
}

type RegisterAssetHook struct {
	Chain uint16 `json:"chain,omitempty"`
	TokenAddr ExternalTokenId `json:"token_address,omitempty"`
}

type ExternalTokenId struct {
	Bytes [32]byte `json:"bytes,omitempty"`
}

func Cw20ContractInstantiateMsg(
	t *testing.T,
	name string,
	symbol string,
	chainID uint16,
	assetAddr string,
	decimals uint8,
	tbContractAddr string,
) string {
	index := 32
	assetAddr32 := [32]byte{}
	for i := len(assetAddr); i > 0; i-- {
		assetAddr32[index-1] = assetAddr[i-1]
		index--
	}

	tbMsg := TbRegisterAssetHookMsg{
		RegisterAssetHook: RegisterAssetHook{
			Chain: chainID,
			TokenAddr: ExternalTokenId{
				Bytes: assetAddr32,
			},
		},
	}
	tbMsgBz, err := json.Marshal(tbMsg)
	require.NoError(t, err)

	msg := Cw20InstantiateMsg{
		Name: name,
		Symbol: symbol,
		AssetChain: chainID,
		AssetAddr: assetAddr32[:],
		Decimals: decimals,
		InitHook: InitHook{
			Msg: tbMsgBz,
			ContractAddr: tbContractAddr,
		},
	}

	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return string(msgBz)

}