import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	Executable,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
	let executable: Executable = {
		command: '/Users/azjezz/mago/mago/target/debug/mago',
		args: ['lsp'],
	};

	let serverOptions: ServerOptions = {
		run: executable,
		debug: executable,
	};

	let clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'php' }],

		synchronize: {
			fileEvents: workspace.createFileSystemWatcher('**/*.php')
		},
	};

	client = new LanguageClient(
		'mago',
		'Mago Language Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server.
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}