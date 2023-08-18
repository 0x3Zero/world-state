use std::process::Command;

pub fn curl(args: Vec<String>) -> String {
  let output = Command::new("curl")
              .args(args)
              .output()
              .expect("Failed to execute the curl command");

  // println!("output: {:?}", output);
  let mut result;
  if output.status.success() {
      // Convert the output bytes to a string
      let response_body = String::from_utf8_lossy(&output.stdout);

      // Print the response body
      // println!("Response body:\n{}", response_body);
      result = response_body;
  } else {
      // If the command failed, print the error message
      let error_message = String::from_utf8_lossy(&output.stderr);
      // println!("Error executing the curl command:\n{}", error_message);
      result = error_message;
  }

  trimmer(result.to_string())
}

pub fn trimmer(text: String) -> String {
  text.replace("\n", "")  // Remove newline characters
  .trim_start()       // Trim leading whitespace
  .trim_end()
  .to_string()
}