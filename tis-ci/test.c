#include <stdio.h>

FILE *fp_output;

int main(int argc, char **argv);

int main_wrapper(int argc, char **argv) {
  /* 1. Prepare the output file. */
  fp_output = fopen("output", "w+");

  /* 2. Run the main test function and remember the return value. */
  int main_retval = main(argc, argv);

  /* 3. Check if the actual test output and the expected output are the same. */
  FILE *fp_expected = fopen("expected", "r");
  int c_output, c_expected;
  int ok = 1;
  rewind(fp_output);

  printf("Checking the output.");
  while (1) {
    c_output = fgetc(fp_output);
    /* If the end of the test output is reached, stop. */
    if (c_output == EOF)
      break;
    /* Each line of the test output should be compared with the same single line
       of the expected output. */
    if (c_output == '\n') {
      rewind(fp_expected);
      printf("Next line.");
      continue;
    }
    c_expected = fgetc(fp_expected);
    /* Each character read from the test output should be the same as
       the character read from the expected output. */
    if (c_expected == c_output) {
      printf("output = %c, expected = %c", c_output, c_expected);
    } else {
      ok = 0;
      printf("output = %c, expected = %c : WRONG!", c_output, c_expected);
    }
  };
  /*@ assert output_as_expected: ok == 1; */
  printf("Done.");

  fclose(fp_expected);
  fclose(fp_output);

  /* 4. Finish up: return the main test function's return value. */
  /*@ assert main_returns_zero: main_retval == 0; */
  return main_retval;
}
