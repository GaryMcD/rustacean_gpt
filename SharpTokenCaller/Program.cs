// Program.cs
using System;
using SharpToken;

class Program {
    static void Main(string[] args) {
        // Create an instance of GptEncoding with the desired encoding
        var encoding = GptEncoding.GetEncoding("cl100k_base");

        // Encode the input string
        var encoded = encoding.Encode(args[0]);

        // Decode the encoded tokens
        var decoded = encoding.Decode(encoded);

        // Print the decoded string
        Console.WriteLine(encoded.Count);
    }
}
