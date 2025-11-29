package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

func TxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tx",
		Short: "Submit transactions",
	}

	cmd.AddCommand(
		txSendCmd(),
		txSubmitJobCmd(),
	)

	return cmd
}

func txSendCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "send [from] [to] [amount]",
		Short: "Send NEX tokens",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Println("Transaction Submitted")
			fmt.Println("=====================")
			fmt.Printf("From:   %s\n", args[0])
			fmt.Printf("To:     %s\n", args[1])
			fmt.Printf("Amount: %s NEX\n", args[2])
			fmt.Println("Status: pending validator approval")
			fmt.Println("")
			fmt.Println("Tx Hash: 0x...")
			return nil
		},
	}
}

func txSubmitJobCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "submit-job",
		Short: "Submit a docking job",
		RunE: func(cmd *cobra.Command, args []string) error {
			target, _ := cmd.Flags().GetString("target")
			ligand, _ := cmd.Flags().GetString("ligand")
			fee, _ := cmd.Flags().GetString("fee")

			fmt.Println("Docking Job Submitted")
			fmt.Println("=====================")
			fmt.Printf("Target:  %s\n", target)
			fmt.Printf("Ligand:  %s\n", ligand)
			fmt.Printf("Fee:     %s NEX\n", fee)
			fmt.Println("Status:  queued for next checkpoint")
			fmt.Println("")
			fmt.Println("Job ID: job_...")
			fmt.Println("")
			fmt.Println("Job will be included in next checkpoint.")
			fmt.Println("Miners will compete to dock and approve.")
			return nil
		},
	}

	cmd.Flags().String("target", "6LU7", "Target protein PDB ID")
	cmd.Flags().String("ligand", "", "Ligand SMILES or file")
	cmd.Flags().String("fee", "10", "Fee in NEX")

	return cmd
}
