package keeper

import (
	"context"

	"tokenfactory/x/tokenmodule/types"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func (k msgServer) TransferTokens(goCtx context.Context, msg *types.MsgTransferTokens) (*types.MsgTransferTokensResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Check if the value exists
	valFound, isFound := k.GetDenom(
		ctx,
		msg.Denom,
	)
	if !isFound {
		return nil, errorsmod.Wrap(sdkerrors.ErrKeyNotFound, "denom does not exist")
	}

	// Checks if the the msg owner is the same as the current owner
	if msg.Owner != valFound.Owner {
		return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "incorrect owner")
	}

	// Ensure reciever address is valid
	_, err := sdk.AccAddressFromBech32(msg.To)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid From address (%s)", err)
	}

	// Convert From address
	fromAddr, err := sdk.AccAddressFromBech32(msg.From)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid From address: %v", err)
	}

	// Convert To address
	toAddr, err := sdk.AccAddressFromBech32(msg.To)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid To address: %v", err)
	}

	// Parse amount
	var coins sdk.Coins
	coins = coins.Add(sdk.NewCoin(msg.Denom, math.NewInt(int64(msg.Amount))))

	// Transfer coins
	err = k.bankKeeper.SendCoins(ctx, fromAddr, toAddr, coins)
	if err != nil {
		return nil, err
	}

	// Emit event
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute("from", msg.From),
			sdk.NewAttribute("to", msg.To),
			sdk.NewAttribute("amount", coins.String()),
		),
	})

	return &types.MsgTransferTokensResponse{}, nil
}
