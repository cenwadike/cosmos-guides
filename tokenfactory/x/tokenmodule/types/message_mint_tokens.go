package types

import (
	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ sdk.Msg = &MsgMintTokens{}

func NewMsgMintTokens(owner string, denom string, amount int32, recipient string) *MsgMintTokens {
	return &MsgMintTokens{
		Owner:     owner,
		Denom:     denom,
		Amount:    amount,
		Recipient: recipient,
	}
}

func (msg *MsgMintTokens) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Owner)
	if err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid owner address (%s)", err)
	}
	return nil
}
