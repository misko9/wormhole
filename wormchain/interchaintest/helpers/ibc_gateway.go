package helpers

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/stretchr/testify/require"
	"github.com/wormhole-foundation/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
)

type IbcGwInstantiateMsg struct {
	TokenBridgeContract string `json:"token_bridge_contract"`
}

func IbcGwContractInstantiateMsg(t *testing.T, tbContract string) string {
	msg := IbcGwInstantiateMsg{
		TokenBridgeContract: tbContract,
	}
	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return string(msgBz)
}

type IbcGwExecute struct {
	CompleteTransferAndConvert CompleteTransferAndConvert `json:"complete_transfer_and_convert"`
}

type CompleteTransferAndConvert struct {
	Vaa []byte `json:"vaa"`
}

// TODO: replace amount's uint64 with big int or equivalent
func CreatePayload3(cfg ibc.ChainConfig, amount uint64, tokenAddr string, tokenChain uint16, recipient string, recipientChain uint16, from []byte, contractPayload []byte) []byte {
	payload := new(bytes.Buffer)
	vaa.MustWrite(payload, binary.BigEndian, uint8(3)) // Payload 3: TransferWithPayload
	payload.Write(make([]byte, 24))
	vaa.MustWrite(payload, binary.BigEndian, amount)

	tokenAddrPadded := vaa.LeftPadBytes(tokenAddr, 32)
	payload.Write(tokenAddrPadded.Bytes())
	vaa.MustWrite(payload, binary.BigEndian, tokenChain)
	
	recipientAddr := MustAccAddressFromBech32(recipient, cfg.Bech32Prefix)
	payload.Write(recipientAddr.Bytes())
	vaa.MustWrite(payload, binary.BigEndian, recipientChain)
	
	payload.Write(from)
	
	payload.Write(contractPayload)

	return payload.Bytes()
}

func IbcGwCompleteTransferAndConvertMsg(t *testing.T, emitterChainID uint16, emitterAddr string, payload []byte, guardians *guardians.ValSet) string {
	emitterBz := [32]byte{}
	eIndex := 32
	for i := len(emitterAddr); i > 0; i-- {
		emitterBz[eIndex-1] = emitterAddr[i-1]
		eIndex--
	}
	v := generateVaa(0, guardians, vaa.ChainID(emitterChainID), vaa.Address(emitterBz), payload)
	vBz, err := v.Marshal()
	require.NoError(t, err)
		
	msg := IbcGwExecute{
		CompleteTransferAndConvert: CompleteTransferAndConvert{
			Vaa: vBz,
		},
	}

	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return string(msgBz)
}

type GatewayIbcTokenBridgePayloadSimple struct {
	Simple Simple `json:"simple"`
}

type Simple struct {
	Chain uint16 `json:"chain"`
	Recipient []byte `json:"recipient"`
	Fee string `json:"fee"` // TODO: should be uint128
	Nonce uint32 `json:"nonce"`
}

func CreateGatewayIbcTokenBridgePayloadSimple(t *testing.T, chainID uint16, recipient string, fee uint64, nonce uint32) []byte {
	msg := GatewayIbcTokenBridgePayloadSimple{
		Simple: Simple{
			Chain: chainID,
			Recipient: []byte(recipient),
			Fee: fmt.Sprint(fee),
			Nonce: nonce,
		},
	}
	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return msgBz
}

type GatewayIbcTokenBridgePayloadContractControlled struct {
	ContractControlled ContractControlled `json:"contract_controlled"`
}

type ContractControlled struct {
	Chain uint16 `json:"chain"`
	Contract []byte `json:"recipient"`
	Payload []byte `json:"payload"`
	Fee uint64 `json:"fee"` // TODO: should be uint128
	Nonce uint32 `json:"nonce"`
}

func CreateGatewayIbcTokenBridgePayloadContract(t *testing.T, chainID uint16, contract string, payload []byte, fee uint64, nonce uint32) []byte {
	msg := GatewayIbcTokenBridgePayloadContractControlled{
		ContractControlled: ContractControlled{
			Chain: chainID,
			Contract: []byte(contract),
			Payload: payload,
			Fee: fee,
			Nonce: nonce,
		},
	}
	msgBz, err := json.Marshal(msg)
	require.NoError(t, err)

	return msgBz
}