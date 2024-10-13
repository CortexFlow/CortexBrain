import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../../')))

from Globals.imports import *
from Connectors.connectors import Connectors
import io
import traceback

from Globals.constants import GLOBAL_VAR


class SyntaxHighlighter(QSyntaxHighlighter):
    def __init__(self, parent=None):
        super(SyntaxHighlighter, self).__init__(parent)

        self.highlightingWords = []  # list for the highlithed words

        # Colore blue
        blue_format = QTextCharFormat()
        blue_format.setForeground(QColor(116, 151, 178))
        # Colore yellow
        yellow_format = QTextCharFormat()
        yellow_format.setForeground(QColor(255, 255, 51))

        # blue keywords
        blue_keywords = ["def", "class", "import", "from", "as", "if", "else", "elif", "return",
                         "while", "for", "in", "break", "continue", "try", "except", "with", "lambda"]
        # yellow keywords
        yellow_keywords = ["\[", "\]", "\(", "\)", "\[\]", "\(\)"]

        for keyword in blue_keywords:
            pattern = QRegularExpression(r'\b' + keyword + r'\b')
            self.highlightingWords.append((pattern, blue_format))

        for keyword in yellow_keywords:
            pattern_y = QRegularExpression(keyword)
            self.highlightingWords.append((pattern_y, yellow_format))

    def highlightBlock(self, text):
        # apply the rules for coloring yellow and blue words
        for pattern, format in self.highlightingWords:
            match_iterator = pattern.globalMatch(text)
            while match_iterator.hasNext():
                match = match_iterator.next()
                self.setFormat(match.capturedStart(),
                               match.capturedLength(), format)


class MainWindow(QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        uic.loadUi(GLOBAL_VAR.APP_SCREEN_UI, self)
        self.setWindowTitle(GLOBAL_VAR.TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.TITLE))

        # insert the buttons
        self.btn_settings.clicked.connect(self.open_settings)
        self.customer_support.clicked.connect(self.custom_support)
        self.donate_btn.clicked.connect(self.donate)
        self.go_home_btn.clicked.connect(self.GoHome)
        self.go_sim_btn.clicked.connect(self.GoSim)
        self.go_datas_btn.clicked.connect(self.GoDatas)
        self.go_progetta_btn.clicked.connect(self.GoProgetta)

        # ------------------------------------------------------------

        # Initialize the text editor
        self.text_editor.setText("Benvenuto nel text editor!")

        # inizialize the syntax highlighter
        self.highlighter = SyntaxHighlighter(self.text_editor.document())

        self.btn_new.clicked.connect(self.newFile)
        self.btn_save.clicked.connect(self.saveFile)
        self.btn_open_file.clicked.connect(self.openFile)
        self.btn_new_text.clicked.connect(self.newFile)
        self.btn_copy_text.clicked.connect(self.copy)
        self.btn_paste_text.clicked.connect(self.paste)
        self.btn_undo_text.clicked.connect(self.undo)
        self.btn_redo_text.clicked.connect(self.redo)
        self.btn_compile_code.clicked.connect(self.compile_code)
        self.btn_run_code.clicked.connect(self.run_code)

        # --------------------------------------------------
        # inizialize CONNECTORS
        self.btn_connectors.clicked.connect(self.open_connectors_window)
        self.connectors_window = None
        self.stackedWidget.setCurrentWidget(self.page_home)

    def highlightCurrentLine(self):
        extraSelections = []

        if not self.text_editor.isReadOnly():
            selection = QTextEdit.ExtraSelection()
            lineColor = QColor(Qt.yellow).lighter(160)
            selection.format.setBackground(lineColor)
            selection.format.setProperty(QTextFormat.FullWidthSelection, True)
            selection.cursor = self.text_editor.textCursor()
            selection.cursor.clearSelection()
            extraSelections.append(selection)

        self.text_editor.setExtraSelections(extraSelections)

    def open_settings(self):
        # change page to the settings page
        self.stackedWidget.setCurrentWidget(self.page_settings)

    def GoHome(self):
        # print("Home button clicked")
        self.stackedWidget.setCurrentWidget(
            self.page_home)  # go to home page

    def GoSim(self):
        self.stackedWidget.setCurrentWidget(
            self.page_sim)  # go to the sim page

    def GoDatas(self):
        self.stackedWidget.setCurrentWidget(
            self.page_datas)  # go to the data page

    def GoProgetta(self):
        # go to the project design page
        self.stackedWidget.setCurrentWidget(self.page_progetta)

    def newFile(self):
        pass

    def custom_support(self):
        pass

    def donate(self):
        pass

    # "save"  file function (TEXT EDITOR FEATURE)
    def saveFile(self):
        if self.current_path is not None:
            filetext = self.text_editor.toPlainText()
            with open(self.current_path, 'w') as f:
                f.write(filetext)
        else:
            self.saveFileAs()

    # "save as" file function (TEXT EDITOR FEATURE)
    def saveFileAs(self):
        pathname = QFileDialog.getSaveFileName(
            self, 'Save file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files(*.txt)')
        filetext = self.text_editor.toPlainText()
        with open(pathname[0], 'w') as f:
            f.write(filetext)
        self.current_path = pathname[0]
        self.setWindowTitle(pathname[0])

    # "open" file function (TEXT EDITOR FEATURE)
    def openFile(self):
        fname = QFileDialog.getOpenFileName(
            self, 'Open file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files (*.txt)')
        self.setWindowTitle(fname[0])
        with open(fname[0], 'r') as f:
            filetext = f.read()
            self.text_editor.setText(filetext)
        self.current_path = fname[0]

    # "undo" file function (TEXT EDITOR FEATURE)

    def undo(self):
        self.text_editor.undo()

    # "redo" file function (TEXT EDITOR FEATURE)
    def redo(self):
        self.text_editor.redo()

    # "copy" file function (TEXT EDITOR FEATURE)
    def copy(self):
        self.text_editor.copy()

    # "paste" file function (TEXT EDITOR FEATURE)
    def paste(self):
        self.text_editor.paste()

    # "compile" code function (TEXT EDITOR FEATURE)
    def compile_code(self):
        code = self.text_editor.toPlainText()
        self.compiler_.clear()  # clear previous output

        output, error = self.compile_code_internal(code)
        if error:
            self.compiler_.append(error)
        else:
            self.compiler_.append("Compiled with no errors")

    # compile code internal--->return no output only for the "compile function" associated with the compile button
    def compile_code_internal(self, code):
        try:
            compiled_code = compile(code, '<string>', 'exec')
            exec_output = {}
            exec(compiled_code, exec_output)
            return None, None  # Return no output and no errors
        except SyntaxError as e:
            return None, f"Errore di sintassi: {e}"  # error handler
        except Exception as e:
            error_message = traceback.format_exc()
            # error message
            return None, f"Errore di esecuzione:\n{error_message}"

    # Run the code from the text editor and display the result in the output window and the compilation result in the compiler window
    def run_code(self):
        # Retrieve the code entered in the text editor
        code = self.text_editor.toPlainText()

        # Clear the compiler window to reset previous messages
        self.compiler_.clear()

        # Compile the code (assuming self.compile_code handles any compilation or syntax checking)
        self.compile_code()

        # Redirect output to a buffer
        buffer = io.StringIO()  # Create a buffer to capture printed output
        # Store the current stdout (console output)
        original_stdout = sys.stdout
        sys.stdout = buffer  # Redirect stdout to the buffer

        try:
            # Dictionary for local variables in the exec environment
            local_vars = {}

            # Execute the code within a controlled environment
            exec(code, {}, local_vars)

            # Get the output from the buffer
            output = buffer.getvalue()

            # If there's output, append it to the compiler side window
            if output:
                self.compiler_side_window.append(output)

        # Catch any exception that occurs during code execution
        except Exception as e:
            # Get the full error traceback and display it in the compiler window
            error_message = traceback.format_exc()
            self.compiler_.append(f"Errore:\n{error_message}")

        # Ensure stdout is always restored, even if an error occurs
        finally:
            sys.stdout = original_stdout  # Restore the original stdout
            buffer.close()  # Close the buffer to free up memory

    # open the connector window
    def open_connectors_window(self):
        # If the Connectors window is already open, bring it to the foreground
        if self.connectors_window is None or not self.connectors_window.isVisible():
            self.connectors_window = Connectors(
                self)  # Create a new Connectors window
        else:
            self.connectors_window.raise_()  # Bring the already open window to the foreground
            self.connectors_window.activateWindow()  # Activates the window

        print("Connection established")

    def on_close(self, event):
        # set the connect window to none when the connectionEstablished window is closed
        self.connectors_window = None
        event.accept()  # Accept the closing event
