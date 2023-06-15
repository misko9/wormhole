package helpers

import (
	"context"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/stretchr/testify/require"
)

func InstantiateContract(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, keyname string, codeId string, message string) (contract string) {
	contractAddr, err := chain.InstantiateContract(ctx, keyname, codeId, message, true)
	require.NoError(t, err)

	return contractAddr
}
