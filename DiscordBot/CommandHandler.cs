﻿#region

using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading.Tasks;
using Discord.Commands;
using Discord.WebSocket;

#endregion

namespace DiscordBot
{
    public class CommandHandler
    {
        private static readonly HttpClient _HttpClient = new HttpClient();
        private readonly DiscordSocketClient _client;
        private readonly CommandService _commands;

        public CommandHandler(DiscordSocketClient client, CommandService commands) {
            _commands = commands;
            _client = client;
        }

        public  Task InstallCommandsAsync() {
            // Hook the MessageReceived event into our command handler
            _client.MessageReceived += HandleCommandAsync;

            // Here we discover all of the command modules in the entry 
            // assembly and load them. Starting from Discord.NET 2.0, a
            // service provider is required to be passed into the
            // module registration method to inject the 
            // required dependencies.
            //
            // If you do not use Dependency Injection, pass null.
            // See Dependency Injection guide for more information.
            return _commands.AddModulesAsync(Assembly.GetEntryAssembly(),
                null);
        }


        private static async Task<string> SendCommand(string name, string command, string url) {
            command = command.Remove(0, 1);
            var args = command.Split(new[] {' '}, StringSplitOptions.RemoveEmptyEntries);

            string cmdContent = string.Empty;
            if (args.Length > 1) {
                cmdContent = "\"" + args.Skip(1).Aggregate((s, n) => s + "\", \"" + n) + "\"";
            }

            var values = new Dictionary<string, string> {
                {"Name", name},
                {"CmdIdentifier", args.First()},
                {"CmdTexts", "[" + cmdContent + "]"}
            };

            var content = new FormUrlEncodedContent(values);

            var response = await _HttpClient.PostAsync(url, content);

            return await response.Content.ReadAsStringAsync();
        }

        private async Task HandleCommandAsync(SocketMessage messageParam) {
            // Don't process the command if it was a system message
            var message = messageParam as SocketUserMessage;
            if (message == null) {
                return;
            }

            // Create a number to track where the prefix ends and the command begins
            var argPos = 0;

            // Determine if the message is a command based on the prefix and make sure no bots trigger commands
            if (!(message.HasCharPrefix('!', ref argPos) ||
                  message.HasMentionPrefix(_client.CurrentUser, ref argPos)) ||
                message.Author.IsBot) {
                return;
            }

            // Create a WebSocket-based command context based on the message
            var context = new SocketCommandContext(_client, message);

            // Execute the command with the command context we just
            // created, along with the service provider for precondition checks.

            // Keep in mind that result does not indicate a return value
            // rather an object stating if the command executed successfully.
            var result = await _commands.ExecuteAsync(
                context,
                argPos,
                null);

            // Optionally, we may inform the user if the command fails
            // to be executed; however, this may not always be desired,
            // as it may clog up the request queue should a user spam a
            // command.

            if (result.Error == CommandError.UnknownCommand) {
                string response = await SendCommand(message.Author.Username, message.Content,
                    "https://kobert.dev/api/dsa/commands");
                //var response = "invalid";
                await context.Channel.SendMessageAsync(response);
            }
            else if (!result.IsSuccess) {
                await context.Channel.SendMessageAsync(result.ErrorReason);
            }
        }
    }
}