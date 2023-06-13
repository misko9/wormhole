package ictest

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"testing"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	"github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/icza/dyno"
	sdk "github.com/cosmos/cosmos-sdk/types"

	interchaintest "github.com/strangelove-ventures/interchaintest/v7"
	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v7/ibc"
	"github.com/strangelove-ventures/interchaintest/v7/testreporter"

	//wormholetypes "github.com/wormhole-foundation/wormchain/x/wormhole/types"

	"github.com/stretchr/testify/require"
	"go.uber.org/zap/zaptest"
)

var (
	pathWormchainGaia   = "wormchain-gaia" // Replace with 2nd cosmos chain supporting wormchain
	genesisWalletAmount = int64(10_000_000)
	votingPeriod = "10s"
	maxDepositPeriod = "10s"
	coinType = "118"
	wormchainConfig = ibc.ChainConfig{
		Type:    "cosmos",
		Name:    "wormchain",
		ChainID: "wormchain-1",
		Images: []ibc.DockerImage{
			{
				Repository: "wormchain",
				Version:    "local",
				UidGid:     "1025:1025",
			},
		},
		Bin:            "wormchaind",
		Bech32Prefix:   "wormhole",
		Denom:          "uworm",
		CoinType:       coinType,
		GasPrices:      "0.00uworm",
		GasAdjustment:  1.8,
		TrustingPeriod: "112h",
		NoHostMount:    false,
		EncodingConfig:         wormchainEncoding(),
	}
	numVals = 2
	numFullNodes = 1
)

// wormchainEncoding registers the Wormchain specific module codecs so that the associated types and msgs
// will be supported when writing to the blocksdb sqlite database.
func wormchainEncoding() *testutil.TestEncodingConfig {
	cfg := cosmos.DefaultEncoding()

	// register custom types
	wasmtypes.RegisterInterfaces(cfg.InterfaceRegistry)

	return &cfg
}

func CreateSingleWormchain(t *testing.T) []ibc.Chain {
	// Create chain factory with wormchain
	wormchainConfig.ModifyGenesis = ModifyGenesis(votingPeriod, maxDepositPeriod)

	cf := interchaintest.NewBuiltinChainFactory(zaptest.NewLogger(t), []*interchaintest.ChainSpec{
		{
			ChainName: "wormchain",
			ChainConfig: wormchainConfig,
			NumValidators: &numVals,
			NumFullNodes: &numFullNodes,
		},
	})

	// Get chains from the chain factory
	chains, err := cf.Chains(t.Name())
	require.NoError(t, err)

	return chains
}

func BuildInitialChain(t *testing.T, chains []ibc.Chain) context.Context {
	// Create a new Interchain object which describes the chains, relayers, and IBC connections we want to use
	ic := interchaintest.NewInterchain()

	for _, chain := range chains {
		ic.AddChain(chain)
	}

	rep := testreporter.NewNopReporter()
	eRep := rep.RelayerExecReporter(t)

	ctx := context.Background()
	client, network := interchaintest.DockerSetup(t)

	err := ic.Build(ctx, eRep, interchaintest.InterchainBuildOptions{
		TestName:          t.Name(),
		Client:            client,
		NetworkID:         network,
		SkipPathCreation:  true,
		BlockDatabaseFile: interchaintest.DefaultBlockDatabaseFilepath(),
	})
	require.NoError(t, err)

	t.Cleanup(func() {
		_ = ic.Close()
	})

	return ctx
}

// Modify the genesis file:
// * Goverance - i.e. voting period
// * Get generated val set
// * Get faucet address
// * Set Guardian Set List using new val set
// * Set Guardian Validator List using new val set
// * Allow list the faucet address
func ModifyGenesis(votingPeriod string, maxDepositPeriod string) func(ibc.ChainConfig, []byte) ([]byte, error) {
	return func(chainConfig ibc.ChainConfig, genbz []byte) ([]byte, error) {
		g := make(map[string]interface{})
		if err := json.Unmarshal(genbz, &g); err != nil {
			return nil, fmt.Errorf("failed to unmarshal genesis file: %w", err)
		}
		
		// Modify gov
		if err := dyno.Set(g, votingPeriod, "app_state", "gov", "voting_params", "voting_period"); err != nil {
			return nil, fmt.Errorf("failed to set voting period in genesis json: %w", err)
		}
		if err := dyno.Set(g, maxDepositPeriod, "app_state", "gov", "deposit_params", "max_deposit_period"); err != nil {
			return nil, fmt.Errorf("failed to set max deposit period in genesis json: %w", err)
		}
		if err := dyno.Set(g, chainConfig.Denom, "app_state", "gov", "deposit_params", "min_deposit", 0, "denom"); err != nil {
			return nil, fmt.Errorf("failed to set min deposit in genesis json: %w", err)
		}
		// Get validators
		var validators [][]byte
		for i := 0; i < numVals; i++ {
			validatorBech32, err := dyno.Get(g, "app_state", "genutil", "gen_txs", i, "body", "messages", 0, "delegator_address")
			if err != nil {
				return nil, fmt.Errorf("failed to get validator pub key: %w", err)
			}
			validatorAccAddr := MustAccAddressFromBech32(validatorBech32.(string), chainConfig.Bech32Prefix).Bytes()
			validators = append(validators, validatorAccAddr)
		}

		// Get faucet address
		faucetAddress, err := dyno.Get(g, "app_state", "auth", "accounts", numVals, "address")

		// Set guardian set list and validators
		guardianSetList := []GuardianSet{}
		guardianSet := GuardianSet{
			Index: 0,
			Keys: [][]byte{},
		}
		guardianValidators := []GuardianValidator{}
		for i := 0; i < numVals; i++ {
			guardianSet.Keys = append(guardianSet.Keys, validators[i])
			guardianValidators = append(guardianValidators, GuardianValidator{
				GuardianKey: validators[i],
				ValidatorAddr: validators[i],
			})
		}
		guardianSetList = append(guardianSetList, guardianSet)
		if err := dyno.Set(g, guardianSetList, "app_state", "wormhole", "guardianSetList"); err != nil {
			return nil, fmt.Errorf("failed to set guardian set list: %w", err)
		}
		if err := dyno.Set(g, guardianValidators, "app_state", "wormhole", "guardianValidatorList"); err != nil {
			return nil, fmt.Errorf("failed to set guardian validator list: %w", err)
		}

		allowedAddresses := []ValidatorAllowedAddress{}
		allowedAddresses = append(allowedAddresses, ValidatorAllowedAddress{
			ValidatorAddress: sdk.MustBech32ifyAddressBytes(chainConfig.Bech32Prefix, validators[0]),
			AllowedAddress: faucetAddress.(string),
			Name: "Faucet",
		})
		if err := dyno.Set(g, allowedAddresses, "app_state", "wormhole", "allowedAddresses"); err != nil {
			return nil, fmt.Errorf("failed to set guardian validator list: %w", err)
		}

		out, err := json.Marshal(g)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal genesis bytes to json: %w", err)
		}
		fmt.Println("Genesis: ", string(out))
		return out, nil
	}
}

func MustAccAddressFromBech32(address string, bech32Prefix string) sdk.AccAddress {
	if len(strings.TrimSpace(address)) == 0 {
		panic("empty address string is not allowed")
	}

	bz, err := sdk.GetFromBech32(address, bech32Prefix)
	if err != nil {
		panic(err)
	}

	err = sdk.VerifyAddressFormat(bz)
	if err != nil {
		panic(err)
	}

	return sdk.AccAddress(bz)
}

// Replace these with reference to x/wormchain/types
type GuardianSet struct {
	Index          uint32   `protobuf:"varint,1,opt,name=index,proto3" json:"index,omitempty"`
	Keys           [][]byte `protobuf:"bytes,2,rep,name=keys,proto3" json:"keys,omitempty"`
	ExpirationTime uint64   `protobuf:"varint,3,opt,name=expirationTime,proto3" json:"expirationTime,omitempty"`
}

type ValidatorAllowedAddress struct {
	// the validator/guardian that controls this entry
	ValidatorAddress string `protobuf:"bytes,1,opt,name=validator_address,json=validatorAddress,proto3" json:"validator_address,omitempty"`
	// the allowlisted account
	AllowedAddress string `protobuf:"bytes,2,opt,name=allowed_address,json=allowedAddress,proto3" json:"allowed_address,omitempty"`
	// human readable name
	Name string `protobuf:"bytes,3,opt,name=name,proto3" json:"name,omitempty"`
}

type GuardianValidator struct {
	GuardianKey   []byte `protobuf:"bytes,1,opt,name=guardianKey,proto3" json:"guardianKey,omitempty"`
	ValidatorAddr []byte `protobuf:"bytes,2,opt,name=validatorAddr,proto3" json:"validatorAddr,omitempty"`
}