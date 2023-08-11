package cli

import (
	"encoding/hex"
	"strconv"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/spf13/cast"
	"github.com/spf13/cobra"
	"github.com/wormhole-foundation/wormchain/x/wormhole/types"
)

var _ = strconv.Itoa(0)

// ScheduleUpgrade will schedule a chain upgrade
func CmdScheduleUpgrade() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "schedule-upgrade [name] [height] [vaa-hex]",
		Short:   "Schedule a chain upgrade",
		Aliases: []string{},
		Args:    cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			
			name := args[0]
			height, err := cast.ToUint64E(args[1])
			if err != nil {
				return err
			}
			vaaBz, err := hex.DecodeString(args[2])
			if err != nil {
				return err
			}
			
			msg := types.MsgScheduleUpgrade{
				Signer:  clientCtx.GetFromAddress().String(),
				Name: name,
				Height: height,
				Vaa: vaaBz,
			}

			if err = msg.ValidateBasic(); err != nil {
				return err
			}
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	return cmd
}