extends VBoxContainer

var url = "ws://127.0.0.1:8888/ws"
var socket = WebSocketPeer.new();

func _ready() -> void:
	socket.connect_to_url(url)

func _process(_delta: float) -> void:
	socket.poll()

	if socket.get_ready_state() == socket.STATE_OPEN:
		while socket.get_available_packet_count():
			var msg = socket.get_packet().get_string_from_utf8()
			%RichTextLabel.text += ('[b]Server:[/b] %s\n' % msg)

func _on_text_submitted(new_text: String) -> void:
	%LineEdit.clear()
	%LineEdit.grab_focus()

	if socket.get_ready_state() == socket.STATE_OPEN:
		socket.send_text('%s: %s' % [%NameEdit.text, new_text])
