package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/wormhole-foundation/wormchain/x/packet-forward-middleware/router/types"
)

// InitGenesis
func (k Keeper) InitGenesis(ctx sdk.Context, state types.GenesisState) {
	k.SetParams(ctx, state.Params)

	// Initialize store refund path for forwarded packets in genesis state that have not yet been acked.
	store := prefix.NewStore(ctx.KVStore(k.storeKey), types.KeyPrefix(types.RefundPacketKeyPrefix))
	for key, value := range state.InFlightPackets {
		bz := k.cdc.MustMarshal(&value)
		store.Set([]byte(key), bz)
	}
}

// ExportGenesis
func (k Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	store := prefix.NewStore(ctx.KVStore(k.storeKey), types.KeyPrefix(types.RefundPacketKeyPrefix))

	inFlightPackets := make(map[string]types.InFlightPacket)

	itr := store.Iterator(nil, nil)
	for ; itr.Valid(); itr.Next() {
		var inFlightPacket types.InFlightPacket
		k.cdc.MustUnmarshal(itr.Value(), &inFlightPacket)
		inFlightPackets[string(itr.Key())] = inFlightPacket
	}
	return &types.GenesisState{Params: k.GetParams(ctx), InFlightPackets: inFlightPackets}
}
