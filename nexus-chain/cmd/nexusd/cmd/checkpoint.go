package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

func CheckpointCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "checkpoint",
		Short: "Checkpoint operations",
	}

	cmd.AddCommand(
		checkpointLatestCmd(),
		checkpointShowCmd(),
		checkpointListCmd(),
	)

	return cmd
}

func checkpointLatestCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "latest",
		Short: "Show latest finalized checkpoint",
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Println("Latest Checkpoint")
			fmt.Println("=================")
			fmt.Println("No checkpoints finalized yet.")
			fmt.Println("")
			fmt.Println("Checkpoints are created every 200 blocks")
			fmt.Println("and require 67% miner approval to finalize.")
			return nil
		},
	}
	return cmd
}

func checkpointShowCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "show [height]",
		Short: "Show checkpoint at height",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			height := args[0]
			fmt.Printf("Checkpoint at height %s\n", height)
			fmt.Println("========================")
			fmt.Println("Status: not found")
			return nil
		},
	}
	return cmd
}

func checkpointListCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List recent checkpoints",
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Println("Recent Checkpoints")
			fmt.Println("==================")
			fmt.Println("Height  Status      Approvals  Finalized")
			fmt.Println("------  ----------  ---------  ---------")
			fmt.Println("(none)")
			return nil
		},
	}
	return cmd
}
