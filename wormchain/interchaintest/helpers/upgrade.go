package helpers

import (
	"context"
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/stretchr/testify/require"
	"github.com/wormhole-foundation/wormchain/interchaintest/guardians"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
)

func ScheduleUpgrade(
	t *testing.T,
	ctx context.Context,
	chain *cosmos.CosmosChain,
	keyName string,
	name string,
	height uint64,
	guardians *guardians.ValSet,
) {

	node := chain.GetFullNode()

	payload := vaa.BodyWormchainScheduleUpgrade{
		Name: name,
		Height: height,
	}
	payloadBz := payload.Serialize()
	v := generateVaa(0, guardians, vaa.ChainID(vaa.GovernanceChain), vaa.Address(vaa.GovernanceEmitter), payloadBz)
	vBz, err := v.Marshal()
	require.NoError(t, err)
	vHex := hex.EncodeToString(vBz)

	_, err = node.ExecTx(ctx, keyName, "wormhole", "schedule-upgrade", name, fmt.Sprint(height), vHex, "--gas", "auto")
	require.NoError(t, err)
}
