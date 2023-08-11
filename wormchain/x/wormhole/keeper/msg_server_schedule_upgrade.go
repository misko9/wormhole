package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"

	"github.com/wormhole-foundation/wormchain/x/wormhole/types"
	"github.com/wormhole-foundation/wormhole/sdk/vaa"
)

func (k msgServer) ScheduleUpgrade(goCtx context.Context, msg *types.MsgScheduleUpgrade) (*types.MsgScheduleUpgradeResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Parse VAA
	v, err := ParseVAA(msg.Vaa)
	if err != nil {
		return nil, err
	}
	// Verify VAA
	action, payload, err := k.VerifyGovernanceVAA(ctx, v, vaa.WasmdModule)
	if err != nil {
		return nil, err
	}

	if vaa.GovernanceAction(action) != vaa.ActionScheduleUpgrade {
		return nil, types.ErrUnknownGovernanceAction
	}
	
	// Validate signer
	_, err = sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "signer")
	}
	ctx.EventManager().EmitEvent(sdk.NewEvent(
		sdk.EventTypeMessage,
		sdk.NewAttribute(sdk.AttributeKeyModule, types.ModuleName),
		sdk.NewAttribute(sdk.AttributeKeySender, msg.Signer),
	))

	// validate the contractAddress in the VAA payload match the ones in the message
	var payloadBody vaa.BodyWormchainScheduleUpgrade
	payloadBody.Deserialize(payload)
	if payloadBody.Name != msg.Name {
		return nil, types.ErrConsensusSetNotUpdatable
	}
	if payloadBody.Height != msg.Height {
		return nil, types.ErrConsensusSetUndefined
	}

	plan := upgradetypes.Plan{
		Name: msg.Name,
		Height: int64(msg.Height),
	}

	k.upgradeKeeper.ScheduleUpgrade(ctx, plan)

	return &types.MsgScheduleUpgradeResponse{}, nil
}
