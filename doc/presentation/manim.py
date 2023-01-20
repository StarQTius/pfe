from manim import *
from numpy import array
from random import choice
from manim_voiceover import VoiceoverScene
from manim_voiceover.services.recorder import RecorderService

NORMAL_FONT = 20

PRIVATE_KEY_PNG = "private_key.png"
PUBLIC_KEY_PNG = "public_key.png"

Text.set_default(color=BLACK)
MathTex.set_default(color=BLACK)
Dot.set_default(color=BLACK)
Rectangle.set_default(color=BLACK)

class Title(VoiceoverScene):
    def construct(self):
        self.camera.background_color = WHITE
        self.set_speech_service(RecorderService(transcription_kwargs={"language": "en"}), create_subcaption=False)
        
        title = Text("CRYSTALS-Dilitihum")
        
        with self.voiceover(text=f"""
            CRYSTALS-Dilithium is a signature scheme. As such, its purpose is to produce signatures for messages in order to garantee its recipient the authenticity and the integrity of the said messages.
        """):
            self.play(Create(title))
        
        self.wait(1.0)

class ValidatedScenario(VoiceoverScene):
    def construct(self):
        self.camera.background_color = WHITE
        self.set_speech_service(RecorderService(transcription_kwargs={"language": "en"}), create_subcaption=False)

        sender = labeled_dude(color=RED, label="Sender")
        recipient = labeled_dude(color=BLUE, label="Recipient")
        party = Group(sender, recipient).arrange(buff=8)
        
        with self.voiceover(text=f"""
            Let's suppose that two individuals want to communicate using CRYSTALS-Dilithium. One of them, {bkmrk('A')} the sender, is going to send a message to the {bkmrk('B')} recipient.
        """):
            self.wait_until_bookmark("A")
            self.play(FadeIn(sender))

            self.wait_until_bookmark("B")
            self.play(FadeIn(recipient))

        challenge_string = random_signature(6);
        message = Text("Incommutable", font_size=NORMAL_FONT)
        signature = Text(f"z = {random_signature(6)}", font_size=NORMAL_FONT)
        challenge = Text(f"c = {challenge_string}", font_size=NORMAL_FONT)
        packet = Group(message, signature, challenge).arrange(DOWN, buff=0.1).next_to(sender, RIGHT)
        
        with self.voiceover(text=f"""
            In order to garantee the recipient that the author of the message is in fact the sender, the latter will compute {bkmrk('A')} a signature from the message right before sending it, as well as {bkmrk('B')} a challenge.
        """):
            self.play(Create(message))
            
            self.wait_until_bookmark("A")
            self.play(Create(signature))
            
            self.wait_until_bookmark("B")
            self.play(Create(challenge))

        self.play(packet.animate.next_to(recipient, LEFT))
        
        calculated_challenge = Text(f"c' = {challenge_string}", color=BLUE, font_size=NORMAL_FONT).next_to(challenge, DOWN)
        validation_label = Text("validated!", font_size=NORMAL_FONT, color=GREEN).next_to(message, UP)
        with self.voiceover(text=f"""
            On reception of the message, the recipient will compute {bkmrk('A')} a challenge from the message, the signature and some more information about the sender. Then, they will compare it with the received challenge. If both match, then the recipient knows that the message comes from no other than the sender, and that the message has not been modified during the transmission. 
        """):
            self.wait_until_bookmark("A")
            self.play(Create(calculated_challenge))

        self.play(Create(validation_label))
        self.wait(1.0)

class RejectedScenario(VoiceoverScene):
    def construct(self):
        self.camera.background_color = WHITE
        self.set_speech_service(RecorderService(transcription_kwargs={"language": "en"}), create_subcaption=False)

        sender = labeled_dude(color=RED, label="Sender")
        recipient = labeled_dude(color=BLUE, label="Recipient")
        party = Group(sender, recipient).arrange(buff=8)
        challenge_string = random_signature(6);
        message = Text("Incommutable", font_size=NORMAL_FONT)
        signature = Text(f"z = {random_signature(6)}", font_size=NORMAL_FONT)
        challenge = Text(f"c = {challenge_string}", font_size=NORMAL_FONT)
        packet = Group(message, signature, challenge).arrange(DOWN, buff=0.1).next_to(sender, RIGHT)
      
        self.play(AnimationGroup(FadeIn(sender), FadeIn(recipient), Create(message), Create(signature), Create(challenge)))

        with self.voiceover(text=f"""
            If an issue happens during the transmission, either because of noise or malicious intent, the recipient will be able to detect this modification. Because the message has been modified, the {bkmrk('A')} calculation of the challenge will yield a different result and the challenges will {bkmrk('B')} no longer match.
        """):
            self.play(packet.animate.move_to([0, 0, 0]))
            self.play(FadeOut(message))

            altered_message = Text("Incomputable", font_size=NORMAL_FONT).move_to(message)
            altered_packet = Group(altered_message, signature, challenge)
            
            self.play(FadeIn(altered_message))
            self.play(altered_packet.animate.next_to(recipient, LEFT))
            
            calculated_signature = Text(f"c' = {random_signature(6)}", color=BLUE, font_size=NORMAL_FONT).next_to(challenge, DOWN)
            rejection_label = Text("rejected!", font_size=NORMAL_FONT, color=RED).next_to(altered_message, UP)
            
            self.wait_until_bookmark("A")
            self.play(Create(calculated_signature))
            
            self.wait_until_bookmark("B")
            self.play(Create(rejection_label))

        integrity_label = Text("Integrity")
        authentication_label = Text("Authenticity")
        label_group = Group(integrity_label, authentication_label).arrange(DOWN).move_to(3 * UP)
        
        with self.voiceover(text=f"""
            Therefore, CRYSTALS-Dilithium enables the recipient to check {bkmrk('A')} the integrity of the messages they receive, which is an important feature of signature schemes.
        """):
            self.wait_until_bookmark("A")
            self.play(Create(integrity_label))

        private_key = key(PRIVATE_KEY_PNG, label="Private key").next_to(sender, DOWN)
        public_key = key(PUBLIC_KEY_PNG, label="Public key").next_to(recipient, DOWN)
        
        with self.voiceover(text=f"""
            It also ensures {bkmrk('A')} the authenticity of the messages. It is possible because both the sender and the recipient possess their own keys. The sender has {bkmrk('B')} a private key, which is used to produce the signature and the challenge. The recipient uses {bkmrk('C')} a public key associated with the sender's private key to check incoming messages.
        """):
            self.wait_until_bookmark("A")
            self.play(Create(authentication_label))
            
            self.wait_until_bookmark("B")
            self.play(FadeIn(private_key))
            
            self.wait_until_bookmark("C")
            self.play(FadeIn(public_key))
        
        self.wait(1.0)

class Lattice(VoiceoverScene):
    def construct(self):
        self.set_speech_service(RecorderService(transcription_kwargs={"language": "en"}), create_subcaption=False)
        self.camera.background_color = WHITE

        master_equation = MathTex(r"Ay + e = z \text{ mod } p")
        condition = MathTex(r'\text{Find }', 'y', r'\text{ and }', 'e', r'\text{ knowing }', r'A', r'\text{, }', 'z', r'\text{ and }', 'p', r'\text{...}')
        conclusion = Text("...hard to solve!", font_size=NORMAL_FONT)
        label_group = Group(master_equation, condition, conclusion).arrange(DOWN).move_to(UP)

        with self.voiceover(text=f"""
            But more than that, CRYSTALS-Dilitihum is a post-quantum cryptography algorithm. That means that it is expected to be resistant to attacks based on quantum computing, and the reason lies in the following problem."""):
            pass

        with self.voiceover(text=f"""
            Given a matrix A, a vector z and an integer p, find two vectors y and e such that their respective norms are the smallest possible. In the general case, this optimization problem is not {bkmrk('B')} easy to solve, and it is predicted to be hard enough for quantum computers as well.
        """):
            self.play(Create(master_equation))
            self.play(Create(condition))

            self.wait_until_bookmark("B")
            self.play(Create(conclusion))

        bdd_problem = Text("Bounded Distance Decoding problem", font_size=30).next_to(master_equation, DOWN)

        pratical_equation = MathTex(r"A", r"s_1", r"+", r"s_2", r"=", r"t", r"\text{ mod } q").move_to(master_equation)

        with self.voiceover(text=f"""
            In pratice, the equation is used in CRYSTALS-Dilitihum like so {bkmrk('A')}. The elements of the private key are {bkmrk('B')} s1 and s2, while the elements of the public are {bkmrk('C')} A and t. Therefore, if the conjecture holds true, there is very little risk of leaking the private key when sharing the public key.
       """):

            self.wait_until_bookmark("A");
            self.play(AnimationGroup(FadeOut(master_equation), FadeIn(pratical_equation)))
        
            self.wait_until_bookmark("B");
            self.play(AnimationGroup(
                pratical_equation[1].animate.set_color(RED),
                pratical_equation[3].animate.set_color(RED),
            ))        
        
            self.wait_until_bookmark("C");
            self.play(AnimationGroup(
                pratical_equation[0].animate.set_color(GREEN),
                pratical_equation[5].animate.set_color(GREEN),
            ))
            
        self.wait(1.0)

def bkmrk(string):
    return f"<bookmark mark='{string}'/>"


def lattice(size, dot_radius, origin, step, shift_step):
    retval = []
    
    (x_origin, y_origin) = origin
    (x_step, y_step) = step
    (width, height) = size
    row_length = 2 * max(int((abs(x_origin) + width) / x_step), int((abs(2 * y_origin) + height) / y_step))
    shift_length = 2 * int((height + width) / shift_step) # Probably not the right formula...

    for shift in range(-shift_length // 2, shift_length // 2):
        for step in range(-row_length // 2, row_length // 2 + 1):
            x = step * x_step + x_origin
            y = step * y_step + shift * shift_step + y_origin
            if (-width <= 2 * x <= width and -height <= 2 * y <= height):
                retval.append(Dot(radius=dot_radius).move_to(x * LEFT + y * UP))

    return Group(*retval)
    

def key(file_path, label):
    SIZE = 0.5
    key = ImageMobject(file_path).scale_to_fit_width(SIZE)
    label = Text(label, font_size=NORMAL_FONT).next_to(key, RIGHT)
    return Group(key, label)


def random_signature(length):
    CHARSET = "azertyuiopqsdfghjklmwxcvbnAZERTYUIOPQSDFGHJKLMWXCVBN0123456789"
    return ''.join([choice(CHARSET) for _ in range(length)])

def dude(color):
    HEAD_OFFSET = 0.7 * UP
    BODY_OFFSET = 0.3 * DOWN
    MASK_OFFSET = 0.3 * DOWN

    HEAD_RADIUS = 0.3
    BODY_WIDTH = 1
    BODY_HEIGHT = 2

    FILL_OPACITY = 0.3

    def head(position):
        return Circle(radius=HEAD_RADIUS, color=color, fill_opacity=FILL_OPACITY).move_to(position)

    def body(position, head_obj):
        circle = Ellipse(width=BODY_WIDTH, height=BODY_HEIGHT).move_to(position)
        mask = Square().move_to(circle.point_at_angle(3 * PI / 2)).shift(MASK_OFFSET)
        return Difference(Difference(circle, mask), head_obj, color=color, fill_opacity=FILL_OPACITY)

    head_obj = head(HEAD_OFFSET);
    return Group(head_obj, body(BODY_OFFSET, head_obj))

def labeled_dude(label, **kwargs):
    dude_obj = dude(**kwargs)
    dude_label = Text(label).next_to(dude_obj, UP)
    return Group(dude_obj, dude_label)
