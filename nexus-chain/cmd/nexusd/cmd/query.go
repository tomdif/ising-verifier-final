package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

func QueryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "query",
		Short: "Query blockchain state",
		Aliases: []string{"q"},
	}

	cmd.AddCommand(
		queryBlockCmd(),
		queryTxCmd(),
		queryAccountCmd(),
	)

	return cmd
}

func queryBlockCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "block [height]",
		Short: "Query block by height",
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			height := "latest"
			if len(args) > 0 {
				height = args[0]
			}
			fmt.Printf("Block %s\n", height)
			fmt.Println("========")
			fmt.Println("Height:     0")
			fmt.Println("Hash:       (genesis)")
			fmt.Println("Txs:        0")
			fmt.Println("Checkpoint: none")
			return nil
		},
	}
}

func queryTxCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "tx [hash]",
		Short: "Query transaction by hash",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Printf("Transaction %s not found\n", args[0])
			return nil
		},
	}
}

func queryAccountCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "account [address]",
		Short: "Query account balance",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Printf("Account: %s\n", args[0])
			fmt.Println("=========")
			fmt.Println("Balance: 0 NEX")
			fmt.Println("Nonce:   0")
			return nil
		},
	}
}
