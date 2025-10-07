extends VBoxContainer

func _on_text_submitted(new_text: String) -> void:
	%RichTextLabel.text += ('[b]User:[/b] %s\n' % new_text)
	%LineEdit.clear()
	%LineEdit.grab_focus()
