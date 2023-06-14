package helpers

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v7/testutil"
	"github.com/stretchr/testify/require"

	"github.com/tendermint/crypto/sha3"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
	"github.com/wormhole-foundation/wormchain/x/wormhole/types"
	"github.com/wormhole-foundation/wormhole/wormchain/interchaintest/guardians"
)

func createWasmStoreCodePayload(wasmBytes []byte) []byte {
	// governance message with sha3 of wasmBytes as the payload
	var hashWasm [32]byte
	keccak := sha3.NewLegacyKeccak256()
	keccak.Write(wasmBytes)
	keccak.Sum(hashWasm[:0])

	gov_msg := types.NewGovernanceMessage(vaa.WasmdModule, byte(vaa.ActionStoreCode), uint16(vaa.ChainIDWormchain), hashWasm[:])
	return gov_msg.MarshalBinary()
}

// wormchaind tx wormhole store [wasm file] [vaa-hex] [flags]
func StoreContract(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, keyName string, fileLoc string, guardians *guardians.ValSet) (codeId string) {
	node := getFullNode(chain)

	_, file := filepath.Split(fileLoc)
	err := node.CopyFile(ctx, fileLoc, file)
	require.NoError(t, err, fmt.Errorf("writing contract file to docker volume: %w", err))

	content, err := os.ReadFile(fileLoc)
	require.NoError(t, err)

	payload := createWasmStoreCodePayload(content)
	v := generateVaa(0, guardians, vaa.ChainID(vaa.GovernanceChain), payload)
	vBz, err := v.Marshal()
	require.NoError(t, err)

	vHex := hex.EncodeToString(vBz)

	stdout, err := node.ExecTx(ctx, keyName, "wormhole", "store", path.Join(node.HomeDir(), file), vHex, "--gas", "auto")
	require.NoError(t, err)
	fmt.Println("Store code stdout: ", stdout)

	err = testutil.WaitForBlocks(ctx, 2, node.Chain)
	require.NoError(t, err)

	stdoutBz, _, err := node.ExecQuery(ctx, "wasm", "list-code", "--reverse")
	require.NoError(t, err)

	res := CodeInfosResponse{}
	err = json.Unmarshal(stdoutBz, &res)
	require.NoError(t, err)

	return res.CodeInfos[0].CodeID
}

type CodeInfo struct {
	CodeID string `json:"code_id"`
}
type CodeInfosResponse struct {
	CodeInfos []CodeInfo `json:"code_infos"`
}